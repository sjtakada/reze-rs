//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Node.
//

use std::char;
use std::rc::Rc;

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

// CLI Node trait.
pub trait CliNode {
    // Return inner.
    fn inner(&self) -> &CliNodeInner;

    // Return ClI token string ref.
    fn token(&self) -> &str;
    
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
    next: Vec<Rc<CliNode>>,
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
            next: Vec::new(),
        }
    }

    fn help(&self) -> &str {
        &self.help
    }

    fn defun(&self) -> &str {
        &self.defun
    }
}

// CLI keyword node
//   static literal
pub struct CliNodeKeyword {
    inner: CliNodeInner,
}

impl CliNodeKeyword {
    pub fn new(id: &str, defun: &str, help: &str, token: &str) -> CliNodeKeyword {
        CliNodeKeyword {
            inner: CliNodeInner::new(id, defun, help, token)
        }
    }
}

impl CliNode for CliNodeKeyword {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
    }
    
    fn collate(&self, input: &str) -> MatchResult {
        let pos = 0;

        if input == self.token() {
            return MatchResult::Success(MatchFlag::Full)
        }

        let t = &self.token()[0..input.len()];
        if input == t {
            return MatchResult::Success(MatchFlag::Partial)
        }

        MatchResult::Failure(pos)
    }

}

// CLI range node
//   integer range to match numeric input.
pub struct CliNodeRange {
    inner: CliNodeInner,
    min: i64,
    max: i64,
}

impl CliNodeRange {
    pub fn new(id: &str, defun: &str, help: &str,
               min: i64, max: i64) -> CliNodeRange {
        let token = format!("<{}-{}>", min, max);

        CliNodeRange {
            inner: CliNodeInner::new(id, defun, help, &token),
            min: min,
            max: max,
        }
    }
}

impl CliNode for CliNodeRange {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
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
    inner: CliNodeInner,
}

impl CliNodeIPv4Prefix {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv4Prefix {
        CliNodeIPv4Prefix {
            inner: CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV4_PREFIX),
        }
    }
}

impl CliNode for CliNodeIPv4Prefix {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
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
    inner: CliNodeInner,
}
    
impl CliNodeIPv4Address {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv4Address {
        CliNodeIPv4Address {
            inner: CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV4_ADDRESS),
        }
    }
}

impl CliNode for CliNodeIPv4Address {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
    }
    
    fn collate(&self, input: &str) -> MatchResult {
        enum State {
            Init,
            Digit,
            Dot,
        };

        let mut pos: u32 = 0;
        let mut val: u32 = 0;
        let mut dots: u8 = 0;
        let mut octets: u8 = 0;
        let mut state = State::Init;

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
                _ => {
                    return MatchResult::Failure(pos)
                }
            }

            pos += 1;
        }

        if (octets != 4) {
            return MatchResult::Success(MatchFlag::Incomplete)
        }

        MatchResult::Success(MatchFlag::Full)
    }
}

// ClI IPv6 Prefix node
//   match IPv6 Prefix (X:X::X:X/M)
pub struct CliNodeIPv6Prefix {
    inner: CliNodeInner,
}

// ClI IPv6 Address node
//   match IPv6 Address (X:X::X:X)
pub struct CliNodeIPv6Address {
    inner: CliNodeInner,
}
    
impl CliNodeIPv6Address {
    pub fn new(id: &str, defun: &str, help: &str) -> CliNodeIPv6Address {
        CliNodeIPv6Address {
            inner: CliNodeInner::new(id, defun, help, CLI_TOKEN_IPV6_ADDRESS),
        }
    }
}

impl CliNode for CliNodeIPv6Address {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
    }
    
    fn collate(&self, input: &str) -> MatchResult {
        enum State {
            Init,
            Xdigit,
            Colon1,
            Colon2,
        };

        let mut pos: u32 = 0;
        let mut first_colon: bool = false;
        let mut double_colon: bool = false;
        let mut xdigits: u32 = 0;
        let mut xdigits_count: u8 = 0;
        let mut colon_count: u8 = 0;
        let mut state = State::Init;

        for c in input.chars() {
            match state {
                State::Init if is_xdigit_or_colon(c) => {
                    match c {
                        ':' => {
                            state = State::Colon1;
                            first_colon = true;
                            colon_count += 1;
                        },
                        _ => {
                            state = State::Xdigit;
                            xdigits += 1;
                        }
                    }
                },
                State::Xdigit if is_xdigit_or_colon(c) => {
                    match c {
                        ':' => {
                            state = State::Colon1;
                            xdigits = 0;
                            xdigits_count += 1;
                            colon_count += 1;
                        },
                        _ => {
                            xdigits += 1;
                        }
                    }
                },
                State::Colon1 if is_xdigit_or_colon(c) => {
                    match c {
                        ':' => {
                            if double_colon {
                                return MatchResult::Failure(pos)
                            }

                            state = State::Colon2;
                            colon_count += 1;
                            double_colon = true;
                        },
                        _ => {
                            if first_colon {
                                return MatchResult::Failure(pos)
                            }

                            state = State::Xdigit;
                            xdigits += 1;
                        }
                    }

                },
                State::Colon2 if c.is_digit(16) => {
                    state = State::Xdigit;
                    xdigits += 1;
                },
                _ => {
                    return MatchResult::Failure(pos)
                }
            }

            if xdigits > 4 || xdigits_count > 8 {
                return MatchResult::Failure(pos)
            }

            if colon_count > 7 && xdigits_count != 7 {
                return MatchResult::Failure(pos)
            }

            pos += 1;
        }

        match state {
            State::Colon2 => {
                if xdigits_count == 7 {
                    return MatchResult::Success(MatchFlag::Full)
                }
                else {
                    return MatchResult::Success(MatchFlag::Incomplete)
                }
            },
            State::Xdigit => {
                if xdigits == 4 {
                    return MatchResult::Success(MatchFlag::Full)
                }
                else {
                    return MatchResult::Success(MatchFlag::Incomplete)
                }
            },
            _ => MatchResult::Success(MatchFlag::Full)
        }
    }
}


//
pub struct CliNodeWord {
    inner: CliNodeInner,
}


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

        assert_eq!(node.token(), "<100-9999>");

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

        assert_eq!(node.token(), "<1-4294967295>");

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

        let result = node.collate("100.100.100.100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("100.100.100.100.");
        assert_eq!(result, MatchResult::Failure(15));

        let result = node.collate("255.255.255.255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

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

        let result = node.collate("1:2:3:4:5:6::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1:2:3:4:5:6:7::");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));
    }
}

