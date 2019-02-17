//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Node.
//

use std::rc::Rc;

use super::tree;
use super::collate::*;

const CLI_TOKEN_IPV4_ADDRESS: &str = "A.B.C.D";
const CLI_TOKEN_IPV4_PREFIX: &str = "A.B.C.D/M";
const CLI_TOKEN_IPV6_ADDRESS: &str = "X:X::X:X";
const CLI_TOKEN_IPV6_PREFIX: &str = "X:X::X:X/M";
const CLI_TOKEN_WORD: &str = "WORD";
const CLI_TOKEN_COMMUNITY: &str = "AA:NN";

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
        if input == self.token() {
            return MatchResult::Success(MatchFlag::Full)
        }

        let t = &self.token()[0..input.len()];
        if input == t {
            return MatchResult::Success(MatchFlag::Partial)
        }

        MatchResult::Failure
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
        let num = match input.parse::<i64>() {
            Ok(num) => num,
            Err(_err) => return MatchResult::Failure,
        };

        if num >= self.min && num <= self.max {
            return MatchResult::Success(MatchFlag::Full)
        }

        MatchResult::Failure
    }
}

//
pub struct CliNodeIPv4Prefix {
    inner: CliNodeInner,
}

//
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
                                return MatchResult::Failure
                            }

                            state = State::Dot;
                            dots += 1;
                        },
                        '0' ... '9' => {
                            val = val * 10 + c.to_digit(10).unwrap();
                            if val > 255 {
                                return MatchResult::Failure
                            }
                        },
                        _ => {
                            return MatchResult::Failure
                        }
                    }
                },
                State::Dot if c.is_digit(10) => {
                    val = c.to_digit(10).unwrap();
                    state = State::Digit;
                    octets += 1;
                },
                _ => {
                    return MatchResult::Failure
                }
            }
        }

        if (octets != 4) {
            return MatchResult::Success(MatchFlag::Incomplete)
        }

        MatchResult::Success(MatchFlag::Full)
    }
}

//
pub struct CliNodeIPv6Prefix {
    inner: CliNodeInner,
}

//
pub struct CliNodeIPv6Address {
    inner: CliNodeInner,
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
        assert_eq!(result, MatchResult::Failure);
    }

    #[test]
    pub fn test_node_range_1() {
        let node = CliNodeRange::new("range", "RANGE", "help", 100i64, 9999i64);

        assert_eq!(node.token(), "<100-9999>");

        let result = node.collate("100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("99");
        assert_eq!(result, MatchResult::Failure);

        let result = node.collate("9999");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("10000");
        assert_eq!(result, MatchResult::Failure);
    }

    #[test]
    pub fn test_node_range_2() {
        let node = CliNodeRange::new("range", "RANGE", "help", 1i64, 4294967295i64);

        assert_eq!(node.token(), "<1-4294967295>");

        let result = node.collate("0");
        assert_eq!(result, MatchResult::Failure);

        let result = node.collate("1");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("4294967295");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("4294967296");
        assert_eq!(result, MatchResult::Failure);
    }

    #[test]
    pub fn test_node_ipv4_address() {
        let node = CliNodeIPv4Address::new("ipv4addr", "IPV4-ADDRESS", "help");

        let result = node.collate("100.100.100.100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("100.100.100.100.");
        assert_eq!(result, MatchResult::Failure);

        let result = node.collate("255.255.255.255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("1.1.1.256");
        assert_eq!(result, MatchResult::Failure);

        let result = node.collate("255");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("1.1.1.");
        assert_eq!(result, MatchResult::Success(MatchFlag::Incomplete));

        let result = node.collate("a.b.c.d");
        assert_eq!(result, MatchResult::Failure);
    }
}

