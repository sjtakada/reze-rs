//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Tree.
//

use std::fmt;

use std::cell::RefCell;
use std::rc::Rc;

use super::node::*;
use super::action::*;

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
    Line,
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
            TokenType::Line => "Line",
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
    top: RefCell<Rc<dyn CliNode>>,

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

    pub fn top(&self) -> Rc<dyn CliNode> {
        self.top.borrow().clone()
    }

    pub fn parent(&self) -> Option<Rc<CliTree>> {
        self.parent.clone()
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn build_command(&self, defun_tokens: &serde_json::Value,
                         command: &serde_json::Value) {
        let defun = &command["defun"];
        if defun.is_string() {
            let privilege = command["privilege"].as_i64().unwrap_or(CLI_DEFAULT_NODE_PRIVILEGE as i64) as u8;
            let mut s = String::from(defun.as_str().unwrap());
            let mut cv: CliNodeVec = Vec::new();
            let mut hv: CliNodeVec = Vec::new();
            let mut tv: CliNodeVec = Vec::new();

            cv.push(self.top.borrow().clone());

            CliTree::build_recursive(&mut cv, &mut hv, &mut tv,
                                     &mut s, defun_tokens, command, privilege);
        }
    }

    fn build_recursive(curr: &mut CliNodeVec, head: &mut CliNodeVec,
                       tail: &mut CliNodeVec, s: &mut String,
                       defun_tokens: &serde_json::Value, command: &serde_json::Value,
                       privilege: u8) -> TokenType {
        let mut is_head = true;

        while s.len() > 0 {
            let (token_type, token) = CliTree::get_defun_token(s);

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
                                                                  s, defun_tokens, command, privilege);
                        token_type == TokenType::VerticalBar
                    } { }

                    if token_type == TokenType::RightBrace {
                        for h in hv {
                            h.inner().set_only_once();
                            CliTree::vector_add_node_each(&mut tv, h.clone());
                        }
                    }
                    else if token_type == TokenType::RightBracket {
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

                    if let Some(new_node) = CliTree::new_node_by_type(token_type, defun_tokens, &token) {
                        let next = match CliTree::find_next_by_node(curr, new_node.clone()) {
                            None => {
                                CliTree::vector_add_node_each(curr, new_node.clone());
                                new_node
                            },
                            Some(next) => next,
                        };

                        // Set privilege
                        if privilege < next.inner().privilege() {
                            next.inner().set_privilege(privilege);
                        }

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

        //
        for node in curr.iter() {
            // Set executable.
            node.set_executable();

            let actions = &command["actions"];
            if actions.is_array() {
                for action in actions.as_array().unwrap() {
                    if action.is_object() {
                        for (key, obj) in action.as_object().unwrap().iter() {
                            match key.as_ref() {
                                "mode" => {
                                    let action = CliActionMode::new(obj);
                                    node.inner().push_action(Rc::new(action));
                                },
                                "remote"|"http" => {
                                    let action = CliActionRemote::new(obj);
                                    node.inner().push_action(Rc::new(action));
                                },
                                "shell" => {
                                },
                                "built-in" => {
                                    let action = CliActionBuiltin::new(obj);
                                    node.inner().push_action(Rc::new(action));
                                },
                                "cond" => {
                                },
                                _ => {
                                    println!("Unknown action {}", key);
                                }
                            }
                        }
                    }
                }
            }

        }

        TokenType::Undef
    }

    // Parse string to return:
    //   TokenType, token and remainder of string.
    fn get_defun_token(s: &mut String) -> (TokenType, Option<String>) {
        let offset;
        let token_type;

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
                                        c == '[' || c == ']' ||
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
                            "LINE" => {
                                token_type = TokenType::Line;
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

    fn vector_add_node_each(curr: &mut CliNodeVec, node: Rc<dyn CliNode>) {
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
    fn new_node_by_type(token_type: TokenType, defun_tokens: &serde_json::Value, token: &str) -> Option<Rc<dyn CliNode>> {
        if defun_tokens[token].is_object() {
            let token_def = defun_tokens[token].as_object().unwrap();
            let id = CliTree::get_str_or(token_def, "id", "<id>");
            let help = CliTree::get_str_or(token_def, "help", "<help>");
            
            let node: Rc<dyn CliNode> = match token_type {
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
                TokenType::Line => Rc::new(CliNodeLine::new(&id, token, &help)),
                //TokenType::Community => CliNodeCommunity::new(&id, token, &help),
                TokenType::Keyword => {
                    match token_def.get("enum") {
                        Some(key) => {
                            Rc::new(CliNodeKeyword::new(&id, token, &help, key.as_str()))
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
        }
        else {
            // Debug
            println!("Unknown defun token {}", token);
            None
        }
    }

    fn find_next_by_node(curr: &CliNodeVec, node: Rc<dyn CliNode>) -> Option<Rc<dyn CliNode>> {
        for c in curr {
            let inner = c.inner();
            let next = inner.next();
            for m in next.iter() {
                if m.inner().display() == node.inner().display() {
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
    pub fn test_tree_get_defun_token_1() {
        let mut s = String::from("ip route IPV4-PREFIX:1 IPV4-ADDRESS:2");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ip");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "route");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::IPv4Prefix);
        assert_eq!(token.unwrap(), "IPV4-PREFIX:1");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::IPv4Address);
        assert_eq!(token.unwrap(), "IPV4-ADDRESS:2");

        assert_eq!(s.len(), 0);
    }

    #[test]
    pub fn test_tree_getcli_token_2() {
        let mut s = String::from("show (ip|ipv6) route");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "show");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::LeftParen);
        assert_eq!(token.unwrap(), "(");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ip");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::VerticalBar);
        assert_eq!(token.unwrap(), "|");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::Keyword);
        assert_eq!(token.unwrap(), "ipv6");

        let (t, token) = CliTree::get_defun_token(&mut s);
        assert_eq!(t, TokenType::RightParen);
        assert_eq!(token.unwrap(), ")");

        let (t, token) = CliTree::get_defun_token(&mut s);
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
        assert_eq!(n0.inner().display(), "a");

        let inner = n0.inner();
        let next = inner.next();
        assert_eq!(next.len(), 1);

        let n1 = &next[0];
        assert_eq!(n1.inner().display(), "b");

        let inner = n1.inner();
        let next = inner.next();
        assert_eq!(next.len(), 2);

        let n20 = &next[0];
        assert_eq!(n20.inner().display(), "c");
        let n21 = &next[1];
        assert_eq!(n21.inner().display(), "d");

        let inner = n20.inner();
        let next = inner.next();
        assert_eq!(next.len(), 3);

        let inner = n21.inner();
        let next = inner.next();
        assert_eq!(next.len(), 3);

        let n30 = &next[0];
        assert_eq!(n30.inner().display(), "e");

        let n31 = &next[1];
        assert_eq!(n31.inner().display(), "f");

        let n32 = &next[2];
        assert_eq!(n32.inner().display(), "g");

        let inner = n32.inner();
        let next = inner.next();
        assert_eq!(next.len(), 4);

        let n40 = &next[0];
        assert_eq!(n40.inner().display(), "e");

        let n41 = &next[1];
        assert_eq!(n41.inner().display(), "f");

        let n42 = &next[2];
        assert_eq!(n42.inner().display(), "g");
        assert_eq!(n42.inner().is_executable(), false);

        let n43 = &next[3];
        assert_eq!(n43.inner().display(), "x");
        assert_eq!(n43.inner().is_executable(), true);
    }
}

