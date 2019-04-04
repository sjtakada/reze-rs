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

// Token Type.
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

// CLI Tree:
//   consists of single CLI definition tree, built per mode.
pub struct CliTree {
    // Mode name.
    mode: String,
    
    // Prompt string.
    prompt: String,

    // Parent CliTree.
    parent: Option<Rc<CliTree>>,

    // Top CliNode.
    top: RefCell<Rc<CliNode>>,

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
            top: RefCell::new(Rc::new(CliNodeDummy::new())),
        }
    }

    pub fn name(&self) -> &str {
        &self.mode
    }

    pub fn top(&self) -> Rc<CliNode> {
        self.top.borrow().clone()
    }

    pub fn parent(&self) -> Option<Rc<CliTree>> {
        self.parent.clone()
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn build_command(&self, tokens: &serde_json::Value,
                         command: &serde_json::Value) {
        let defun = &command["defun"];
        if defun.is_string() {
            let mut s = String::from(defun.as_str().unwrap());
            let mut cv: CliNodeVec = Vec::new();
            let mut hv: CliNodeVec = Vec::new();
            let mut tv: CliNodeVec = Vec::new();

            cv.push(self.top.borrow().clone());

            CliTree::build_recursive(&mut cv, &mut hv, &mut tv,
                                     &mut s, tokens, command);
        }
    }

    fn build_recursive(curr: &mut CliNodeVec, head: &mut CliNodeVec,
                       tail: &mut CliNodeVec, s: &mut String,
                       tokens: &serde_json::Value, command: &serde_json::Value) -> TokenType {
        let mut is_head = true;

        while s.len() > 0 {
            let (token_type, token) = CliTree::get_cli_token(s);

            match token_type {
                TokenType::LeftParen |
                TokenType::LeftBracket |
                TokenType::LeftBrace => {
                    let mut hv: CliNodeVec = Vec::new();
                    let mut tv: CliNodeVec = Vec::new();
                    let mut token_type;

                    while {
                        let mut cv = curr.clone();
                        token_type = CliTree::build_recursive(&mut cv, &mut hv, &mut tv,
                                                                  s, tokens, command);
                        token_type == TokenType::VerticalBar
                    } { }

                    if token_type == TokenType::RightBrace || token_type == TokenType::RightBracket {
                        for h in hv {
                            CliTree::vector_add_node_each(&mut tv, h.clone());
                        }
                    }

                    curr.clear();
                    curr.append(&mut tv);
                },
                TokenType::RightParen |
                TokenType::RightBracket |
                TokenType::RightBrace |
                TokenType::VerticalBar => {
                    for c in curr {
                        tail.push(c.clone());
                    }
                    return token_type
                },
                TokenType::Undef => {
                    println!("Undefined token type");
                },
                _ => {
                    let token = token.unwrap();

                    if let Some(new_node) = CliTree::new_node_by_type(token_type, tokens, &token) {
                        let next = match CliTree::find_next_by_same_token(curr, &token) {
                            None => {
                                CliTree::vector_add_node_each(curr, new_node.clone());
                                new_node
                            },
                            Some(next) => next,
                        };

                        // TBD: hidden
                        
                        curr.clear();
                        curr.push(next.clone());
                        
                        if is_head {
                            head.push(next.clone());
                            is_head = false;
                        }
                    }
                }
            }
        }

        for n in curr {
            n.set_executable();
        }

        TokenType::Undef
    }

    // Parse string to return:
    //   TokenType, token and remainder of string.
    fn get_cli_token(s: &mut String) -> (TokenType, Option<String>) {
        let mut offset = 0;
        let mut token_type = TokenType::Undef;

        // trim whitespaces at beginning.
        let len = s.trim_start().len();
        if len != s.len() {
            s.replace_range(..s.len() - len, "");
        }

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
                        offset = s.find(|c: char|
                                        c == '(' || c == ')' ||
                                        c == '{' || c == '}' ||
                                        c == '[' || c == '[' ||
                                        c == '|' || c == ' ').unwrap_or(s.len());

                        let word = &s[..offset];
                        let p = word.find(':').unwrap_or(word.len());

                        match &s[..p] {
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
                return (TokenType::Undef, None);
            }
        }

        let token = String::from(&s[0..offset]);
        s.replace_range(..offset, "");

        (token_type, Some(token))
    }

    fn vector_add_node_each(curr: &mut CliNodeVec, node: Rc<CliNode>) {
        for c in curr {
            let inner = c.inner();
            let mut next = inner.next();
            next.push(node.clone());
        }
    }

    // Utility function to get string out of JSON value or default.
    fn get_str_or<'a>(map: &'a serde_json::map::Map<String, serde_json::Value>, key: &str, def: &'a str) -> &'a str {
        match map.get(key) {
            Some(value) => {
                value.as_str().unwrap_or(def)
            },
            None => def
        }
    }

    // Return CLI Node by TokenType.
    fn new_node_by_type(token_type: TokenType, tokens: &serde_json::Value, token: &str) -> Option<Rc<CliNode>> {
        match tokens.get(token) {
            Some(token_def) if token_def.is_object() => {

                let token_def = tokens[token].as_object().unwrap();
                let id = CliTree::get_str_or(token_def, "id", "<id>");
                let help = CliTree::get_str_or(token_def, "help", "<help>");
                
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
                    TokenType::Keyword => {
                        match token_def.get("enum") {
                            Some(enum_key) => {
                                Rc::new(CliNodeKeyword::new(&id, token, &help, enum_key.as_str()))
                            },
                            None => {
                                Rc::new(CliNodeKeyword::new(&id, token, &help, None))
                            }
                        }
                    },
                    _ => {
                        return None;
                    }
                };

                Some(node)
            },
            _ => None
        }
    }

    fn find_next_by_same_token(curr: &CliNodeVec, token: &str) -> Option<Rc<CliNode>> {
        for c in curr {
            let inner = c.inner();
            let next = inner.next();
            for m in next.iter() {
                if m.inner().token() == token {
                    return Some(m.clone());
                }
            }
        }

        None
    }

    pub fn sort(&self) {
        let top = self.top.borrow().clone();
        top.sort_recursive();
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
    pub fn test_tree_get_cli_token_1() {
        let mut s = String::from("ip route IPV4-PREFIX:1 IPV4-ADDRESS:2");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ip");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "route");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::IPv4Prefix);
        assert_eq!(token.unwrap(), "IPV4-PREFIX:1");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::IPv4Address);
        assert_eq!(token.unwrap(), "IPV4-ADDRESS:2");

        assert_eq!(s.len(), 0);
    }

    #[test]
    pub fn test_tree_getcli_token_2() {
        let mut s = String::from("show (ip|ipv6) route");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "show");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::LeftParen);
        assert_eq!(token.unwrap(), "(");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ip");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::VerticalBar);
        assert_eq!(token.unwrap(), "|");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ipv6");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::RightParen);
        assert_eq!(token.unwrap(), ")");

        let (t, token) = CliTree::get_cli_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "route");

        assert_eq!(s.len(), 0);
    }

    #[test]
    pub fn test_tree_build_recursive() {
        let json_str = r##"
{
  "dummy-cmd": {
    "token": {
      "a": {
        "id": "0",
        "type": "keyword",
        "help": "help"
      },
      "b": {
        "id": "1",
        "type": "keyword",
        "help": "help"
      },
      "c": {
        "id": "2.0",
        "type": "keyword",
        "help": "help"
      },
      "d": {
        "id": "2.1",
        "type": "keyword",
        "help": "help"
      },
      "e": {
        "id": "3.0",
        "type": "keyword",
        "help": "help"
      },
      "f": {
        "id": "3.1.0",
        "type": "keyword",
        "help": "help"
      },
      "g": {
        "id": "3.1.1",
        "type": "keyword",
        "help": "help"
      },
      "h": {
        "id": "3.2",
        "type": "keyword",
        "help": "help"
      },
      "x": {
        "id": "4",
        "type": "keyword",
        "help": "help"
      }
    },
    "command": [
      {
        "defun": "a b (c|d) {e|f|g} x",
        "mode": [
        ]
      }
    ]
  }
} "##;

        let tree = CliTree::new("mode".to_string(), ">".to_string(), None);

        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let json = json["dummy-cmd"].as_object().unwrap();
        let commands: &Vec<serde_json::Value> = json["command"].as_array().unwrap();

        for command in commands {
            tree.build_command(&json["token"], command);
        }

        let top = tree.top.borrow();
        let inner = top.inner();
        let next = inner.next();
        assert_eq!(next.len(), 1);

        let n0 = &next[0];
        assert_eq!(n0.inner().token(), "a");

        let inner = n0.inner();
        let next = inner.next();
        assert_eq!(next.len(), 1);

        let n1 = &next[0];
        assert_eq!(n1.inner().token(), "b");

        let inner = n1.inner();
        let next = inner.next();
        assert_eq!(next.len(), 2);

        let n20 = &next[0];
        assert_eq!(n20.inner().token(), "c");
        let n21 = &next[1];
        assert_eq!(n21.inner().token(), "d");

        let inner = n20.inner();
        let next = inner.next();
        assert_eq!(next.len(), 3);

        let inner = n21.inner();
        let next = inner.next();
        assert_eq!(next.len(), 3);

        let n30 = &next[0];
        assert_eq!(n30.inner().token(), "e");

        let n31 = &next[1];
        assert_eq!(n31.inner().token(), "f");

        let n32 = &next[2];
        assert_eq!(n32.inner().token(), "g");

        let inner = n32.inner();
        let next = inner.next();
        assert_eq!(next.len(), 4);
    }
}

