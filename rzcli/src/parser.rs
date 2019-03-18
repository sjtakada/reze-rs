//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Parser
//

use std::cell::Cell;
use std::rc::Rc;
use std::collections::HashMap;

use serde_json;

use super::error::CliError;
use super::tree::CliTree;
use super::node::CliNode;
use super::collate::*;

//
type CliNodeMatchStateTuple = (Rc<CliNode>, MatchResult);
type CliNodeMatchStateVec = Vec<CliNodeMatchStateTuple>;
type CliNodeTokenTuple = (Rc<CliNode>, String);
type CliNodeTokenVec = Vec<CliNodeTokenTuple>;

//
#[derive(PartialEq, Copy, Clone)]
enum CliExecResult {
    Complete,
    Incomplete,
    Ambiguous,
    Unrecognized,
}

pub struct CliParser {
    // Initial input string TB removed
    input: String,

    // Input length.
    input_len: usize,

    // Current input string.
    line: String,

    // Last token.
    token: String,

    // Matched length.
    matched_len: usize,

    // Vector of pair of CliNode and MatchState.
    matched_vec: Cell<CliNodeMatchStateVec>,

    // Return value, whether or not it hits executable command.
    is_cmd: bool,
}

impl CliParser {
    pub fn new(buf: &str) -> CliParser {
        let input = String::from(buf);
        let mut line = String::from(" ");
        line.push_str(buf);

        CliParser {
            input: input,
            input_len: buf.len(),
            line: line,
            token: String::new(),
            matched_len: 0usize,
            matched_vec: Cell::new(Vec::new()),
            is_cmd: false,
        }
    }

    fn line(&self) -> &str {
        &self.line
    }

    // Return number of matched in current matched.
    fn num_matched(&mut self) -> usize {
        self.matched_vec.get_mut().len()
    }

    // Return next node in matched, must be only one in the Vec.
    fn get_next(&mut self) -> Rc<CliNode> {
        assert!(self.num_matched() == 1);

        self.matched_vec.get_mut()[0].0.clone()
    }

    // Adjust length 1.
    fn matched_len_inc(&mut self) {
        self.matched_len += 1;
    }

    fn set_matched_vec(&mut self, node: Rc<CliNode>) {
        self.matched_vec.replace(
            node.inner().next().iter()
                .map(|n| (n.clone(), MatchResult::Success(MatchFlag::Partial)))
                .collect());
    }

    fn filter_hidden(&mut self) {
        self.matched_vec.get_mut().retain(|n| !n.0.inner().is_hidden());
    }

    // Return true if space exists at the beginning and trim it, or return false.
    fn trim_start(&mut self) -> bool {
        let s = &self.line;
        let len = s.trim_start().len();
        if len != self.line.len() {
            self.line.replace_range(..self.line.len() - len, "");
            return true
        }

        false
    }

    // Get first token, and update line with remainder.
    fn get_token(&mut self) -> Option<String> {
        if self.line.len() == 0 {
            None
        }
        else {
            let pos = match self.line.find(|c: char| c.is_whitespace()) {
                Some(pos) => pos,
                None => self.line.len(),
            };

            let token = String::from(&self.line[..pos]);
            self.line.replace_range(..pos, "");
            Some(token)
        }
    }

    // Save token to state.
    fn save_token(&mut self, token: &str) {
        self.token = String::from(token);
    }

    //
    fn filter_matched(&mut self, mut limit: MatchFlag) {
        let min = self.matched_vec.get_mut().iter()
            .filter_map(|n|
                        match n.1 {
                            MatchResult::Success(flag) => Some(flag),                            
                            _ => None
                        })
            .min();

        if let Some(min) = min {
            let min = if min < limit {
                min
            }
            else {
                limit
            };
            
            self.matched_vec.get_mut().retain(|n| match n.1 {
                MatchResult::Success(flag) if flag <= limit => true,
                _ => false
            });
        };
    }

    // Given input on current CliNode, update matched_vec.
    fn match_token(&mut self, token: &str, curr: Rc<CliNode>) {
        let inner = curr.inner();
        self.matched_vec.replace(
            inner.next().iter()
                .filter_map(|n|
                            match n.collate(token) {
                                MatchResult::Success(flag) =>
                                    Some((n.clone(), MatchResult::Success(flag))),
                                _ => None
                            })
                .collect());
    }

    // try match shorter string in line.
    fn match_shorter(&mut self, curr: Rc<CliNode>, token: String) {
        let mut len = token.len();
        let parsed_len = self.input.len() - self.line.len();

        while len > 0 {
            let sub_token = &token[..len];

            self.set_matched_vec(curr.clone());
            self.filter_hidden();

            self.match_token(sub_token, curr.clone());
            if self.num_matched() > 0 {
                break;
            }

            len -= 1;
        }

        self.matched_len = parsed_len + len;
    }

    fn parse(&mut self, node: Rc<CliNode>) -> CliExecResult {
        let mut curr = node;

        loop {
            self.matched_vec.get_mut().clear();  // TBD: is it needed?
            if !self.trim_start() {
                break;
            }

            self.set_matched_vec(curr.clone());
            self.filter_hidden();

            let token = match self.get_token() {
                Some(token) => token,
                None => break,
            };

            self.save_token(&token);
            self.match_token(&token, curr.clone());

            // At the end of input.
            if !self.line.is_empty()  {
                self.filter_matched(MatchFlag::Partial);

                // No match, try shorter to find one.
                if self.num_matched() == 0 {
                    self.match_shorter(curr.clone(), token.clone());
                    if curr.inner().next().len() == 0 {
                        self.matched_len_inc();
                    }

                    return CliExecResult::Unrecognized
                }
                // Matched more than one, ambiguous.
                else if self.num_matched() > 1 {
                    return CliExecResult::Ambiguous
                }
                // Matched one, move to next node.
                else {
                    let next = self.get_next();
                    return self.parse(next);
                }
            }

            // Not yet at the end of input, but no match.
            if self.num_matched() == 0 {
                self.match_shorter(curr.clone(), token.clone());
                if curr.inner().next().len() == 0 {
                    self.matched_len_inc();
                }

                return CliExecResult::Unrecognized
            }
            else if self.num_matched() == 1 {
                curr = self.get_next();
            }

            break;
        }

        self.is_cmd = curr.inner().is_cmd();
        if self.is_cmd {
            CliExecResult::Complete
        }
        else {
            CliExecResult::Incomplete
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_token() {
        let mut p = CliParser::new("show ip ospf interface");

        let ret = p.trim_start();
        assert_eq!(ret, true);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(p.line(), String::from(" ip ospf interface"));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(p.line(), String::from(" ospf interface"));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(p.line(), String::from(" interface"));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("interface")));
        assert_eq!(p.line(), String::from(""));
    }

    #[test]
    pub fn test_get_token_space() {
        let mut p = CliParser::new(" show   ip ospf ");

        let ret = p.trim_start();
        assert_eq!(ret, true);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(p.line(), String::from("   ip ospf "));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(p.line(), String::from(" ospf "));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(p.line(), String::from(" "));

        let ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, None);
        assert_eq!(p.line(), String::from(""));
    }

    #[test]
    pub fn test_parse() {
        let json_str = r##"
{
  "ospf-show-cmd": {
    "token": {
      "show": {
        "id": "0",
        "type": "keyword",
        "help": "help"
      },
      "ip": {
        "id": "1.0",
        "type": "keyword",
        "help": "help"
      },
      "ipv6": {
        "id": "1.1",
        "type": "keyword",
        "help": "help"
      },
      "ospf": {
        "id": "2",
        "type": "keyword",
        "help": "help"
      },
      "interface": {
        "id": "3.0",
        "type": "keyword",
        "help": "help"
      },
      "neighbor": {
        "id": "3.1",
        "type": "keyword",
        "help": "help"
      },
      "database": {
        "id": "3.2",
        "type": "keyword",
        "help": "help"
      }
    },
    "command": [
      {
        "defun": "show (ip|ipv6) ospf (interface|neighbor|database)",
        "mode": [
        ]
      }
    ]
  }
} "##;

        let tree = CliTree::new("mode".to_string(), ">".to_string(), None);

        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let json = json["ospf-show-cmd"].as_object().unwrap();
        let commands: &Vec<serde_json::Value> = json["command"].as_array().unwrap();

        for command in commands {
            tree.build_command(&json["token"], command);
        }

        let mut p = CliParser::new("show ip ospf");
        let result = p.parse(tree.top());
    }
}
