//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Parser
//

use std::fmt;
use std::cell::Cell;
use std::cell::RefMut;
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
pub enum CliExecResult {
    Complete,
    Incomplete,
    Ambiguous,
    Unrecognized,
}

impl CliExecResult {
    pub fn to_string(&self) -> &str {
        match *self {
            CliExecResult::Complete => "Complete",
            CliExecResult::Incomplete => "Incomplete",
            CliExecResult::Ambiguous => "Ambiguous",
            CliExecResult::Unrecognized => "Unrecognized",
        }
    }
}

impl fmt::Debug for CliExecResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// CLI Parser:
//   Store intermediate state of parser.
//
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
    command: bool,
}

impl CliParser {
    pub fn new(buf: &str) -> CliParser {
        let input = String::from(buf);
        let mut line = String::from(" ");
        line.push_str(buf);

        CliParser {
            input,
            input_len: buf.len(),
            line,
            token: String::new(),
            matched_len: 0usize,
            matched_vec: Cell::new(Vec::new()),
            command: false,
        }
    }

    //
    pub fn matched_vec(&self) -> CliNodeMatchStateVec {
        self.matched_vec.replace(Vec::new())
    }

    // Return current remaining line string.
    fn line(&self) -> &str {
        &self.line
    }

    // Return number of matched in current matched.
    fn num_matched(&mut self) -> usize {
        self.matched_vec.get_mut().len()
    }

    // Return next node in matched.
    fn get_next(&mut self) -> Rc<CliNode> {
        assert!(self.num_matched() == 1);

        self.matched_vec.get_mut()[0].0.clone()
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

    // Saved token size.
    pub fn saved_token_size(&self) -> usize {
        self.token.len()
    }

    //
    fn filter_matched(&mut self, limit: MatchFlag) {
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
        //let inner = curr.inner();
        self.matched_vec.replace(
            curr.inner().next().iter()
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

    pub fn parse(&mut self, curr: Rc<CliNode>) -> CliExecResult {
        let mut curr = curr;

        loop {
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

            // Not yet at the end of input.
            if !self.line.is_empty()  {
                self.filter_matched(MatchFlag::Partial);

                // No match, try shorter to find one.
                if self.num_matched() == 0 {
                    self.match_shorter(curr.clone(), token.clone());
                    if curr.inner().next().len() == 0 {
                        self.matched_len += 1;
                    }

                    return CliExecResult::Unrecognized;
                }

                // Matched more than one, ambiguous.
                if self.num_matched() > 1 {
                    self.filter_matched(MatchFlag::Full);
                    if self.num_matched() == 1 {
                        let next = self.get_next();
                        return self.parse(next);
                    }

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
                    self.matched_len += 1;
                }

                return CliExecResult::Unrecognized
            }
            else if self.num_matched() == 1 {
                curr = self.get_next();
            }

            break;
        }

        self.command = curr.inner().is_executable();
        if self.command {
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
        assert_eq!(result, CliExecResult::Incomplete);

        let mut p = CliParser::new("show x");
        let result = p.parse(tree.top());
        assert_eq!(result, CliExecResult::Unrecognized);

        let mut p = CliParser::new("show ip xy");
        let result = p.parse(tree.top());
        assert_eq!(result, CliExecResult::Unrecognized);

        let mut p = CliParser::new("s i o i");
        let result = p.parse(tree.top());
        assert_eq!(result, CliExecResult::Ambiguous);

        let mut p = CliParser::new("s ip o i");
        let result = p.parse(tree.top());
        assert_eq!(result, CliExecResult::Complete);

        let mut p = CliParser::new("s ipv o i");
        let result = p.parse(tree.top());
        assert_eq!(result, CliExecResult::Complete);
    }
}
