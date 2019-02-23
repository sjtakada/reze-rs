//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Tree.
//

use std::rc::Rc;
use std::fmt;

use super::node;
use super::collate;

#[derive(PartialEq)]
pub enum TokenType {
    Undef,
    WhiteSpace,
    VerticalBar,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    IPv4Prefix,
    IPv4Address,
    IPv6Prefix,
    IPv6Address,
    Range,
    Word,
    Community,
    Array,
    Keyword,
}

impl TokenType {
    pub fn to_string(&self) -> &str {
        match *self {
            TokenType::Undef => "Undefined",
            TokenType::WhiteSpace => "WhiteSpace",
            TokenType::VerticalBar => "VerticalBar",
            TokenType::LeftParen => "LeftParen",
            TokenType::RightParen => "RightParen",
            TokenType::LeftBracket => "LeftBracket",
            TokenType::RightBracket => "RightBracket",
            TokenType::LeftBrace => "LeftBrace",
            TokenType::RightBrace => "RightBrace",
            TokenType::IPv4Prefix => "IPv4Prefix",
            TokenType::IPv4Address => "IPv4Address",
            TokenType::IPv6Prefix => "IPv6Prefix",
            TokenType::IPv6Address => "IPv6Address",
            TokenType::Range => "Range",
            TokenType::Word => "Word",
            TokenType::Community => "Community",
            TokenType::Array => "Array",
            TokenType::Keyword => "Keyword",
        }
    }
}

impl fmt::Debug for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

//
pub struct CliTree {
    // Mode name.
    //mode: String,
    
    // Prompt.
    prompt: String,

    // Parent CliTree.
    parent: Option<Rc<CliTree>>,

    // Exit to finish flag.
    // exit_to_finish: bool;

    // Exit to end flag.
    // exit_to_end: bool;
}

impl CliTree {
    pub fn new(prompt: String, parent: Option<Rc<CliTree>>) -> CliTree {
        CliTree {
            prompt: prompt,
            parent: parent,
        }
    }

    pub fn build_command() {
    }

    // Parse string to return:
    //   TokenType, token and remainder of string.
    fn get_cli_token(s: &str) -> (TokenType, &str, &str) {
        let mut offset = 0;
        let mut token_type = TokenType::Undef;

        // trim whitespaces at beginning.
        let s = s.trim_left();

        match s.chars().next() {
            Some(c) => {
                match c {
                    '|' => {
                        offset = 1;
                        token_type = TokenType::VerticalBar;
                    },
                    '(' => {
                        offset = 1;
                        token_type = TokenType::LeftParen;
                    },
                    ')' => {
                        offset = 1;
                        token_type = TokenType::RightParen;
                    },
                    '[' => {
                        offset = 1;
                        token_type = TokenType::LeftBracket;
                    },
                    ']' => {
                        offset = 1;
                        token_type = TokenType::RightBracket;
                    },
                    '{' => {
                        offset = 1;
                        token_type = TokenType::LeftBrace;
                    },
                    '}' => {
                        offset = 1;
                        token_type = TokenType::RightBrace;
                    },
                    _ => {
                        offset = match s.find(|c: char|
                                              c == '(' || c == ')' ||
                                              c == '{' || c == '}' ||
                                              c == '[' || c == '[' ||
                                              c == '|' || c == ' ') {
                            Some(i) => i,
                            None => s.len()
                        };

                        match &s[0..offset] {
                            "IPV4-PREFIX" => {
                                token_type = TokenType::IPv4Prefix;
                            },
                            "IPV4-ADDRESS" => {
                                token_type = TokenType::IPv4Address;
                            },
                            "IPV6-PREFIX" => {
                                token_type = TokenType::IPv6Prefix;
                            },
                            "IPV6-ADDRESS" => {
                                token_type = TokenType::IPv6Address;
                            },
                            "RANGE" => {
                                token_type = TokenType::Range;
                            },
                            "WORD" => {
                                token_type = TokenType::Word;
                            },
                            "COMMUNITY" => {
                                token_type = TokenType::Community;
                            },
                            /*
                            "METRIC-OFFSET" => {
                                token_type = TokenType::;
                            },
                            "TIME" => {
                                token_type = TokenType::;
                            },
                            "MONTH" => {
                                token_type = TokenType::;
                            },
                             */
                            "ARRAY" => {
                                token_type = TokenType::Array;
                            },
                            _ => {
                                token_type = TokenType::Keyword;
                            }
                        }
                    },
                }
            },
            None => {
                // caller should check token_type.
                return (token_type, s, s);
            }
        }

        let token = &s[0..offset];
        let s = &s[offset..];

        (token_type, token, s)
    }

    fn build_recursive() {
    }

    fn vector_add_node_each() {
    }

    fn new_node_by_type() {
    }

    fn find_next_by_node() {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_cli_token_1() {
        let s = String::from("ip route IPV4-PREFIX IPV4-ADDRESS");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "ip");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "route");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::IPv4Prefix);
        assert_eq!(token, "IPV4-PREFIX");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::IPv4Address);
        assert_eq!(token, "IPV4-ADDRESS");

        assert_eq!(s.len(), 0);
    }

    #[test]
    pub fn test_get_cli_token_2() {
        let s = String::from("show (ip|ipv6) route");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "show");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::LeftParen);
        assert_eq!(token, "(");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "ip");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::VerticalBar);
        assert_eq!(token, "|");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "ipv6");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::RightParen);
        assert_eq!(token, ")");

        let (t, token, s) = CliTree::get_cli_token(&s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token, "route");

        assert_eq!(s.len(), 0);
    }

}
