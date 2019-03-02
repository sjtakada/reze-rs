//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Tree.
//

use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

use super::node::*;
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
    mode: String,
    
    // Prompt.
    prompt: String,

    // Parent CliTree.
    parent: Option<Rc<CliTree>>,

    // Top CliNode.
    top: Option<RefCell<Rc<CliNode>>>,

    // Exit to finish flag.
    // exit_to_finish: bool;

    // Exit to end flag.
    // exit_to_end: bool;
}

impl CliTree {
    pub fn new(mode: String, prompt: String, parent: Option<Rc<CliTree>>) -> CliTree {
        CliTree {
            mode: mode,
            prompt: prompt,
            parent: parent,
            top: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.mode
    }

    pub fn parent(&self) -> Option<Rc<CliTree>> {
        self.parent.clone()
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn build_command(&self, tokens: &serde_json::Value,
                         command: &serde_json::Value) {
//        let tokens = tokens.as_object().unwrap();
        let defun = &command["defun"];
        if !defun.is_string() {
            let s = defun.as_str().unwrap();
            let mut cv: CliNodeVec = Vec::new();
            let mut hv: CliNodeVec = Vec::new();
            let mut tv: CliNodeVec = Vec::new();

            self.build_recursive(&mut cv, &mut hv, &mut tv, s, tokens, command);
        }
    }

    // Create new Vec and clone reference to each CliNode.
    //fn clone_clinode_vec(vec: &CliNodeVec) -> CliNodeVec {
    //vec.iter().map(|v| v.clone()).collect()
    //}

    fn build_recursive(&self, curr: &mut CliNodeVec, head: &mut CliNodeVec, tail: &mut CliNodeVec,
                       s: &str, tokens: &serde_json::Value, command: &serde_json::Value) -> TokenType {
        //
        let mut next: Rc<CliNode>;
        let mut node: Rc<CliNode>;
        let mut is_head = true;

        while s.len() > 0 {
            let (token_type, token, s) = CliTree::get_cli_token(s);
            match token_type {
                TokenType::LeftParen | TokenType::LeftBracket | TokenType::LeftBrace => {
                    let mut hv: CliNodeVec = Vec::new();
                    let mut tv: CliNodeVec = Vec::new();

                    while {
                        let mut cv = curr.clone();
                        let token_type = self.build_recursive(&mut cv, &mut hv, &mut tv, s, tokens, command);

                        token_type == TokenType::VerticalBar
                    } { }

                    if token_type == TokenType::RightBrace || token_type == TokenType::RightBracket {
                        
                    }
                },
                TokenType::RightParen | TokenType::RightBracket | TokenType::RightBrace |
                TokenType::VerticalBar => {
                },
                _ => {
                }
            }
        }

        TokenType::Undef
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

//    fn vector_add_node_each(curr: &mut CliNodeVec, node: Rc<CliNode>) {
//        for c in curr {
//            let mut inner = c.inner();
//            inner.push_next(node.clone());
//        }
//    }

    // TBD: Err()
    fn new_node_by_type(token_type: TokenType, tokens: &serde_json::Value, token: &str) -> Option<Rc<CliNode>> {
        if !tokens[token].is_object() {
            return None
        }
        let token_def = tokens[token].as_object().unwrap();

        if !token_def["id"].is_string() || !token_def["help"].is_string() {
            let id = token_def["help"].as_str().unwrap();
            let help = token_def["help"].as_str().unwrap();

            let node: Rc<CliNode> = match token_type {
                TokenType::IPv4Prefix => Rc::new(CliNodeIPv4Prefix::new(&id, token, &help)),
                TokenType::IPv4Address => Rc::new(CliNodeIPv4Address::new(&id, token, &help)),
                TokenType::IPv6Prefix => Rc::new(CliNodeIPv6Prefix::new(&id, token, &help)),
                TokenType::IPv6Address => Rc::new(CliNodeIPv6Address::new(&id, token, &help)),
                TokenType::Range => {
                    if token_def["range"].is_array() {
                        let range = token_def["range"].as_array().unwrap();
                        let min = range[0].as_i64().unwrap();
                        let max = range[1].as_i64().unwrap();

                        Rc::new(CliNodeRange::new(&id, token, &help, min, max))
                    }
                    else {
                        Rc::new(CliNodeRange::new(&id, token, &help, 0, 1))
                    }
                },
                TokenType::Word => Rc::new(CliNodeWord::new(&id, token, &help)),
                //TokenType::Community => CliNodeCommunity::new(&id, token, &help),
                TokenType::Keyword => Rc::new(CliNodeKeyword::new(&id, token, &help, "TBD")),
                _ => {
                    panic!("unknown type");
                }
            };

            return Some(node)
        }

        None
    }

    fn find_next_by_node(vec: &CliNodeVec, new_node: Rc<CliNode>) -> Option<Rc<CliNode>> {
        for node in vec {
            let inner = node.inner();
            let next = inner.next();
            for m in next.iter() {
                if m.inner().token() == new_node.inner().token() {
                    return Some(m.clone());
                }
            }
        }

        None
    }
}

impl fmt::Debug for CliTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.mode.to_string())
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