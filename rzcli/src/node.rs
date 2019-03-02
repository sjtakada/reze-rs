//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Node.
//

use std::char;
use std::rc::Rc;
use std::cell::Ref;
use std::cell::RefMut;
use std::cell::RefCell;
use std::collections::HashMap;

use super::tree;
use super::collate::*;

const CLI_TOKEN_IPV4_ADDRESS: &str = "A.B.C.D";
const CLI_TOKEN_IPV4_PREFIX: &str = "A.B.C.D/M";
const CLI_TOKEN_IPV6_ADDRESS: &str = "X:X::X:X";
const CLI_TOKEN_IPV6_PREFIX: &str = "X:X::X:X/M";
const CLI_TOKEN_WORD: &str = "WORD";
const CLI_TOKEN_COMMUNITY: &str = "AA:NN";

// utilities
pub fn is_xdigit_or_colon(c: char) -> bool {
    if c.is_digit(16) || c == ':' {
        return true
    }
    return false
}

pub fn is_xdigit_or_colon_or_slash(c: char) -> bool {
    if c.is_digit(16) || c == ':' || c == '/' {
        return true
    }
    return false
}

pub type CliNodeVec = Vec<Rc<CliNode>>;

// CLI Node trait.
pub trait CliNode {
    // Return inner.
    fn inner(&self) -> RefMut<CliNodeInner>;

    // Return match result and flag against input.
    fn collate(&self, input: &str) -> MatchResult;

    // Return help string ref.
    //fn help(&self) -> &str;

    // Return defun token.
    //fn defun_token(&self) -> &str;
}

// Common field for CliNode
pub struct CliNodeInner {
    // Node ID.
    id: String,

    // Defun token.
    defun: String,

    // Help string.
    help: String,

    // CLI token.
    token: String,

    // Node vector is sorted.
    sorted: bool,

    // Hidden flag.
    hidden: bool,

    // Next candidate.
    next: RefCell<CliNodeVec>,
}

impl CliNodeInner {
    pub fn new(id: &str, defun: &str, help: &str, token: &str) -> CliNodeInner {
        CliNodeInner {
            id: String::from(id),
            defun: String::from(defun),
            help: String::from(help),
            token: String::from(token),
            sorted: false,
            hidden: false,
            next: RefCell::new(Vec::new()),
        }
    }

    pub fn defun(&self) -> &str {
        &self.defun
    }

    pub fn help(&self) -> &str {
        &self.help
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn push_next(&mut self, node: Rc<CliNode>) {
        self.next.borrow_mut().push(node);
    }

    pub fn next(&self) -> RefMut<CliNodeVec> {
        self.next.borrow_mut()
    }
}

// CLI keyword node
//   static literal
pub struct CliNodeKeyword {
    inner: RefCell<CliNodeInner>,
}

impl CliNodeKeyword {
    pub fn new(id: &str, defun: &str, help: &str, token: &str) -> CliNodeKeyword {
        CliNodeKeyword {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, token))
        }
    }
}

impl CliNode for CliNodeKeyword {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        let pos = 0;

        if input == self.inner().token {
            return MatchResult::Success(MatchFlag::Full)
        }

        let t = &self.inner().token[0..input.len()];
        if input == t {
            return MatchResult::Success(MatchFlag::Partial)
        }

        MatchResult::Failure(pos)
    }

}

// CLI range node
//   integer range to match numeric input.
pub struct CliNodeRange {
    inner: RefCell<CliNodeInner>,
    min: i64,
    max: i64,
}

impl CliNodeRange {
    pub fn new(id: &str, defun: &str, help: &str,
               min: i64, max: i64) -> CliNodeRange {
        let token = format!("<{}-{}>", min, max);

        CliNodeRange {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, &token)),
            min: min,
            max: max,
        }
    }
}

impl CliNode for CliNodeRange {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        let pos = 0;

        let num = match input.parse::<i64>() {
            Ok(num) => num,
            Err(_err) => return MatchResult::Failure(pos),
        };

        if num >= self.min && num <= self.max {
            return MatchResult::Success(MatchFlag::Full)
        }

        MatchResult::Failure(pos)
    }
}

// CLI IPv4 Prefix node
//   match IPv4 Prefix (A.B.C.D/M)
pub struct CliNodeIPv4Prefix {
    inner: RefCell<CliNodeInner>,
}

impl CliNodeIPv4Prefix {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv4Prefix {
        CliNodeIPv4Prefix {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV4_PREFIX)),
        }
    }
}

impl CliNode for CliNodeIPv4Prefix {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        enum State {
            Init,
            Digit,
            Dot,
            Slash,
            Plen,
        };

        let mut pos: u32 = 0;
        let mut val: u32 = 0;
        let mut dots: u8 = 0;
        let mut octets: u8 = 0;
        let mut state = State::Init;
        let mut plen: u32 = 0;

        for c in input.chars() {
            match state {
                State::Init if c.is_digit(10) => {
                    state = State::Digit;
                    octets += 1;
                    val = c.to_digit(10).unwrap();
                },
                State::Digit => {
                    match c {
                        '.' => {
                            if dots == 3 {
                                return MatchResult::Failure(pos)
                            }
                            state = State::Dot;
                            dots += 1;
                        },
                        '/' => {
                            state = State::Slash;
                        },
                        '0' ... '9' => {
                            val = val * 10 + c.to_digit(10).unwrap();
                            if val > 255 {
                                return MatchResult::Failure(pos)
                            }
                        },
                        _ => {
                            return MatchResult::Failure(pos)
                        },
                    }
                },
                State::Dot if c.is_digit(10) => {
                    val = c.to_digit(10).unwrap();
                    state = State::Digit;
                    octets += 1;
                },
                State::Slash if c.is_digit(10) => {
                    state = State::Plen;
                    plen = c.to_digit(10).unwrap();
                },
                State::Plen if c.is_digit(10) => {
                    plen = plen * 10 + c.to_digit(10).unwrap();
                    if plen > 32 {
                        return MatchResult::Failure(pos)
                    }
                },
                _ => {
                    return MatchResult::Failure(pos)
                },
            }

            pos += 1;
        }

        match state {
            State::Plen => MatchResult::Success(MatchFlag::Full),
            _ => MatchResult::Success(MatchFlag::Incomplete)
        }
    }
}

// CLI IPv4 Address node
//   match IPv4 Address (A.B.C.D)
pub struct CliNodeIPv4Address {
    inner: RefCell<CliNodeInner>,
}
    
impl CliNodeIPv4Address {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv4Address {
        CliNodeIPv4Address {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV4_ADDRESS)),
        }
    }
}

impl CliNode for CliNodeIPv4Address {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        #[derive(Copy, Clone, PartialEq)]
        enum State {
            Init,
            Digit,
            Dot,
            Unknown,
        }

        #[derive(PartialEq)]
        enum Token {
            Digit,
            Dot,
            Unknown,
        }

        let mut pos: u32 = 0;
        let mut val: u32 = 0;
        let mut dots: u8 = 0;
        let mut octets: u8 = 0;
        let mut state = State::Init;
        let mut next_state = State::Unknown;

        for c in input.chars() {
            next_state = State::Unknown;
            let token = match c {
                '0' ... '9' => Token::Digit,
                '.' => Token::Dot,
                _ => break,
            };

            // State machine.
            next_state = match (state, token) {
                // Init
                (State::Init, Token::Digit) => State::Digit,
                // Digit
                (State::Digit, Token::Digit) => State::Digit,
                (State::Digit, Token::Dot) if dots == 3 => break,
                (State::Digit, Token::Dot) => State::Dot,
                // Dot
                (State::Dot, Token::Digit) => State::Digit,
                // Error
                (_, _) => break,
            };

            if next_state == State::Digit {
                val = val * 10 + c.to_digit(10).unwrap();
                if val > 255 {
                    next_state = State::Unknown;
                    break;
                }
            }

            if state != next_state {
                if next_state == State::Digit {
                    octets += 1;
                }
                else if next_state == State::Dot {
                    val = 0;
                    dots += 1;
                }

                state = next_state;
            }

            pos += 1;
        }

        if next_state == State::Unknown {
            MatchResult::Failure(pos)
        }
        else if octets != 4 {
            MatchResult::Success(MatchFlag::Incomplete)
        }
        else if octets == 4 && (val != 0 && val <= 25) {
            MatchResult::Success(MatchFlag::Incomplete)
        }
        else {
            MatchResult::Success(MatchFlag::Full)
        }
    }
}

// ClI IPv6 Prefix node
//   match IPv6 Prefix (X:X::X:X/M)
pub struct CliNodeIPv6Prefix {
    inner: RefCell<CliNodeInner>,
}

impl CliNodeIPv6Prefix {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv6Prefix {
        CliNodeIPv6Prefix {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV6_PREFIX)),
        }
    }
}

impl CliNode for CliNodeIPv6Prefix {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        #[derive(Copy, Clone, PartialEq)]
        enum State {
            Init,
            FirstColon,
            Xdigit,
            Colon,
            DoubleColon,
            Slash,
            PrefixLen,
            Unknown,
        }

        #[derive(PartialEq)]
        enum Token {
            Colon,
            Xdigit,
            Slash,
            PlenDigit,
            Unknown,
        }

        let mut pos: u32 = 0;
        let mut double_colon: bool = false;
        let mut xdigits: u32 = 0;
        let mut xdigits_count: u8 = 0;
        let mut state = State::Init;
        let mut plen: u32 = 0;

        for c in input.chars() {
            let mut next_state = state;
            let token = match state {
                State::Slash | State::PrefixLen => {
                    match c {
                        '0' ... '9' => Token::PlenDigit,
                        _ => Token::Unknown,
                    }
                },
                _ => {
                    match c {
                        '0' ... '9' | 'a' ... 'f' | 'A' ... 'F'
                            => Token::Xdigit,
                        ':' => Token::Colon,
                        '/' => Token::Slash,
                        _   => Token::Unknown,
                    }
                },
            };

            if token == Token::Xdigit {
                xdigits += 1;
                if xdigits > 4 {
                    state = State::Unknown;
                    break;
                }
            }

            // State machine.
            next_state = match (state, token) {
                // Init
                (State::Init, Token::Colon) => State::FirstColon,
                (State::Init, Token::Xdigit) => State::Xdigit,
                // FirstColon
                (State::FirstColon, Token::Colon) => State::DoubleColon,
                // Xdigit
                (State::Xdigit, Token::Colon) if xdigits_count == 8 => State::Unknown,
                (State::Xdigit, Token::Colon) => State::Colon,
                (State::Xdigit, Token::Xdigit) => State::Xdigit,
                (State::Xdigit, Token::Slash) => State::Slash,
                // Colon
                (State::Colon, Token::Colon) if double_colon => State::Unknown,
                (State::Colon, Token::Colon) => {
                    double_colon = true;
                    State::DoubleColon
                },
                (State::Colon, Token::Xdigit) => State::Xdigit,
                // DoubleColon
                (State::DoubleColon, Token::Xdigit) => State::Xdigit,
                (State::DoubleColon, Token::Slash) => State::Slash,
                // Slash / PrefixLen
                (_, Token::PlenDigit) => {
                    plen = plen * 10 + c.to_digit(10).unwrap();
                    if plen > 128 {
                        State::Unknown
                    }
                    else {
                        State::PrefixLen
                    }
                },
                // Error
                (_, _) => State::Unknown,
            };

            if state != next_state {
                if next_state == State::Unknown {
                    state = State::Unknown;
                    break;
                }

                if next_state == State::Xdigit {
                    xdigits_count += 1;
                }
                else if state == State::Xdigit {
                    xdigits = 0;
                }

                state = next_state
            }

            pos += 1;
        }

        match state {
            State::PrefixLen if plen >= 13 || plen == 0 =>
                MatchResult::Success(MatchFlag::Full),
            State::Unknown =>
                MatchResult::Failure(pos),
            _ => 
                MatchResult::Success(MatchFlag::Incomplete),
        }
    }
}


// ClI IPv6 Address node
//   match IPv6 Address (X:X::X:X)
pub struct CliNodeIPv6Address {
    inner: RefCell<CliNodeInner>,
}
    
impl CliNodeIPv6Address {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv6Address {
        CliNodeIPv6Address {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV6_ADDRESS)),
        }
    }
}

impl CliNode for CliNodeIPv6Address {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        #[derive(Copy, Clone, PartialEq)]
        enum State {
            Init,
            FirstColon,
            Xdigit,
            Colon,
            DoubleColon,
            Unknown,
        }

        #[derive(PartialEq)]
        enum Token {
            Colon,
            Xdigit,
        }

        let mut pos: u32 = 0;
        let mut double_colon: bool = false;
        let mut xdigits: u32 = 0;
        let mut xdigits_count: u8 = 0;
        let mut state = State::Init;

        for c in input.chars() {
            let mut next_state = state;
            let token = match c {
                '0' ... '9' | 'a' ... 'f' | 'A' ... 'F'
                    => Token::Xdigit,
                ':' => Token::Colon,
                _   => {
                    state = State::Unknown;
                    break;
                }
            };

            if token == Token::Xdigit {
                xdigits += 1;
                if xdigits > 4 {
                    state = State::Unknown;
                    break;
                }
            }

            // State machine.
            next_state = match (state, token) {
                // Init
                (State::Init, Token::Colon) => State::FirstColon,
                (State::Init, Token::Xdigit) => State::Xdigit,
                // FirstColon
                (State::FirstColon, Token::Colon) => {
                    double_colon = true;
                    State::DoubleColon
                },
                // Xdigit
                (State::Xdigit, Token::Colon) if xdigits_count == 8 => State::Unknown,
                (State::Xdigit, Token::Colon) => State::Colon,
                (State::Xdigit, Token::Xdigit) => state,
                // Colon
                (State::Colon, Token::Colon) if double_colon => State::Unknown,
                (State::Colon, Token::Colon) => {
                    double_colon = true;
                    State::DoubleColon
                },
                (State::Colon, Token::Xdigit) => State::Xdigit,
                // DoubleColon
                (State::DoubleColon, Token::Xdigit) => State::Xdigit,
                // Error
                (_, _) => State::Unknown,
            };

            if state != next_state {
                if next_state == State::Unknown {
                    state = State::Unknown;
                    break;
                }

                if next_state == State::Xdigit {
                    xdigits_count += 1;
                }
                else if state == State::Xdigit {
                    xdigits = 0;
                }

                state = next_state
            }

            pos += 1;
        }

        match state {
            State::Init | State::FirstColon =>
                MatchResult::Success(MatchFlag::Incomplete),
            State::Xdigit if xdigits == 4 && (double_colon || xdigits_count == 8) =>
                MatchResult::Success(MatchFlag::Full),
            State::Xdigit =>
                MatchResult::Success(MatchFlag::Incomplete),
            State::Colon => 
                MatchResult::Success(MatchFlag::Incomplete),
            State::DoubleColon if xdigits_count == 7 =>
                MatchResult::Success(MatchFlag::Full),
            State::DoubleColon =>
                MatchResult::Success(MatchFlag::Incomplete),
            State::Unknown =>
                MatchResult::Failure(pos),
        }
    }
}


//
pub struct CliNodeWord {
    inner: RefCell<CliNodeInner>,
}

impl CliNodeWord {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeWord {
        CliNodeWord {
            inner: RefCell::new(CliNodeInner::new(id, defun, help, CLI_TOKEN_WORD)),
        }
    }
}

impl CliNode for CliNodeWord {
    fn inner(&self) -> RefMut<CliNodeInner> {
        self.inner.borrow_mut()
    }

    fn collate(&self, input: &str) -> MatchResult {
        MatchResult::Success(MatchFlag::Incomplete)
    }
}

//
// Unis tests.
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_node_keyword() {
        let node = CliNodeKeyword::new("show", "show", "help", "show");

        let result = node.collate("show");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("sho");
        assert_eq!(result, MatchResult::Success(MatchFlag::Partial));

        let result = node.collate("shop");
        assert_eq!(result, MatchResult::Failure(0));
    }

    #[test]
    pub fn test_node_range_1() {
        let node = CliNodeRange::new("range", "RANGE", "help", 100i64, 9999i64);

        assert_eq!(node.inner().token(), "<100-9999>");

        let result = node.collate("100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("99");
        assert_eq!(result, MatchResult::Failure(0));

        let result = node.collate("9999");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("10000");
        assert_eq!(result, MatchResult::Failure(0));
    }

    #[test]
    pub fn test_node_range_2() {
        let node = CliNodeRange::new("range", "RANGE", "help", 1i64, 4294967295i64);

        assert_eq!(node.inner().token(), "<1-4294967295>");

        let result = node.collate("0");
        assert_eq!(result, MatchResult::Failure(0));

        let result = node.collate("1");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("4294967295");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("4294967296");
        assert_eq!(result, MatchResult::Failure(0));
    }

    #[test]
    pub fn test_node_ipv4_address() {
        let node = CliNodeIPv4Address::new("ipv4addr", "IPV4-ADDRESS", "help");

        let result = node.collate("0.0.0.0");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("100.100.100.100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("100.100.100.100.");
        assert_eq!(result, MatchResult::Failure(15));

        let result = node.collate("255.255.255.255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1.1.1.256");
        assert_eq!(result, MatchResult::Failure(8));

        let result = node.collate("1.1.1.25");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1.1.26");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1.1.");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1..1");
        assert_eq!(result, MatchResult::Failure(4));

        let result = node.collate("a.b.c.d");
        assert_eq!(result, MatchResult::Failure(0));
    }

    #[test]
    pub fn test_node_ipv4_prefix() {
        let node = CliNodeIPv4Prefix::new("ipv4addr", "IPV4-PREFIX", "help");

        let result = node.collate("100.100.100.100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("100.100.100.100.");
        assert_eq!(result, MatchResult::Failure(15));

        let result = node.collate("255.255.255.255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1.1.256");
        assert_eq!(result, MatchResult::Failure(8));

        let result = node.collate("255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1.1.");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1..1");
        assert_eq!(result, MatchResult::Failure(4));

        let result = node.collate("a.b.c.d");
        assert_eq!(result, MatchResult::Failure(0));

        let result = node.collate("10.10.10.10/");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("10.10.10.10//");
        assert_eq!(result, MatchResult::Failure(12));

        let result = node.collate("0.0.0.0/0");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("10.10.10.10/32");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("10.10.10.10/33");
        assert_eq!(result, MatchResult::Failure(13));
    }

    #[test]
    pub fn test_node_ipv6_address() {
        let node = CliNodeIPv6Address::new("ipv6addr", "IPV6-ADDRESS", "help");

        let result = node.collate("::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("::1");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("2001::1234");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("2001:::1234");
        assert_eq!(result, MatchResult::Failure(6));

        let result = node.collate("2001::123x");
        assert_eq!(result, MatchResult::Failure(9));

        let result = node.collate("2001::12345");
        assert_eq!(result, MatchResult::Failure(10));

        let result = node.collate("1234:5678:90ab:cdef:1234:5678:90ab:cdef");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1234:5678:90ab:cdef:1234:5678:90ab:cdef:");
        assert_eq!(result, MatchResult::Failure(39));

        let result = node.collate("1:2:3:4:5:6:7:8");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7:8888");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1:2:3:4:5:6:7:8:");
        assert_eq!(result, MatchResult::Failure(15));

        let result = node.collate("1:2:3:4:5:6::8");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6::8888");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1::2::3");
        assert_eq!(result, MatchResult::Failure(5));

        let result = node.collate("1:2:3:4:5:6:");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7:");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));
    }

    #[test]
    pub fn test_node_ipv6_prefix() {
        let node = CliNodeIPv6Prefix::new("ipv6prefix", "IPV6-PREFIX", "help");

        let result = node.collate("::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("::/0");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("::1");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("2001::1234");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("2001:::1234");
        assert_eq!(result, MatchResult::Failure(6));

        let result = node.collate("2001::123x");
        assert_eq!(result, MatchResult::Failure(9));

        let result = node.collate("2001::12345");
        assert_eq!(result, MatchResult::Failure(10));

        let result = node.collate("1234:5678:90ab:cdef:1234:5678:90ab:cdef");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1234:5678:90ab:cdef:1234:5678:90ab:cdef:");
        assert_eq!(result, MatchResult::Failure(39));

        let result = node.collate("1:2:3:4:5:6:7:8");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7:8888");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7:8:");
        assert_eq!(result, MatchResult::Failure(15));

        let result = node.collate("1:2:3:4:5:6::8");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6::8888");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1::2::3");
        assert_eq!(result, MatchResult::Failure(5));

        let result = node.collate("1:2:3:4:5:6::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7::/64");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1:2:3:4:5:6:7::/128");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1:2:3:4:5:6:7::/12");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7::/13");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1:2:3:4:5:6:7:://");
        assert_eq!(result, MatchResult::Failure(16));

        let result = node.collate("1:2:3:4:5:6:7::/:");
        assert_eq!(result, MatchResult::Failure(16));

        let result = node.collate("1:2:3:4:5:6:7::/1f");
        assert_eq!(result, MatchResult::Failure(17));
    }
}

