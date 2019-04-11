//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Parser.
//

use std::fmt;
use std::cell::Cell;
use std::rc::Rc;

use serde_json;

use super::error::CliError;
use super::node::CliNode;
use super::collate::*;

// Type aliases.
type CliNodeMatchStateTuple = (Rc<CliNode>, MatchResult);
type CliNodeMatchStateVec = Vec<CliNodeMatchStateTuple>;
type CliNodeTokenTuple = (Rc<CliNode>, String);
type CliNodeTokenVec = Vec<CliNodeTokenTuple>;

// CLI Execution Result.
#[derive(PartialEq, Copy, Clone)]
pub enum ExecResult {
    Complete,
    Incomplete,
    Ambiguous,
    Unrecognized(usize),
}

impl ExecResult {
    pub fn to_string(&self) -> String {
        match *self {
            ExecResult::Complete => "Complete".to_string(),
            ExecResult::Incomplete => "Incomplete".to_string(),
            ExecResult::Ambiguous => "Ambiguous".to_string(),
            ExecResult::Unrecognized(pos) => format!("Unrecognized at {}", pos),
        }
    }
}

impl fmt::Debug for ExecResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// CLI Parser:
//   Store intermediate state of parser.
//
pub struct CliParser {
    // Initial input string.
    input: String,

    // Current input string.
    line: String,

    // Last token.
    token: String,

    // Matched length.
    matched_len: usize,

    // Vector of pair of CliNode and MatchState.
    matched_vec: Cell<CliNodeMatchStateVec>,

    // Vecot of pair of CliNode and input token.
    node_token_vec: Cell<CliNodeTokenVec>,

    // Return value, whether or not it hits executable command.
    executable: bool,
}

impl CliParser {
    // Constructor.
    pub fn new() -> CliParser {
        CliParser {
            input: String::new(),
            line: String::new(),
            token: String::new(),
            matched_len: 0usize,
            matched_vec: Cell::new(Vec::new()),
            node_token_vec: Cell::new(Vec::new()),
            executable: false,
        }
    }

    // Set input and reset state.
    pub fn init(&mut self, input: &str) {
        self.input = String::from(input);
        self.reset_line();
    }

    // Reset line and other parser state.
    pub fn reset_line(&mut self) {
        self.line = String::from(" ");
        self.line.push_str(&self.input);
        self.matched_len = 0;
        self.matched_vec.replace(Vec::new());
        self.node_token_vec.replace(Vec::new());
        self.executable = false;
    }

    // Return current parser position.
    pub fn current_pos(&self) -> usize {
        self.input.len() - self.line.len()
    }

    // Return current matched vec and set it empty.
    pub fn matched_vec(&self) -> CliNodeMatchStateVec {
        self.matched_vec.replace(Vec::new())
    }

    // Return current matched string len.
    pub fn matched_len(&self) -> usize {
        self.matched_len
    }

    // Return reference to current remaining line string.
    fn line(&self) -> &str {
        &self.line
    }

    // Return number of matched in current matched.
    fn num_matched(&mut self) -> usize {
        self.matched_vec.get_mut().len()
    }

    // Return candidate in matched.
    fn get_candidate(&mut self) -> Rc<CliNode> {
        assert!(self.num_matched() == 1);

        self.matched_vec.get_mut()[0].0.clone()
    }

    // Return candidate's MatchResult.
    fn get_candidate_result(&mut self) -> MatchResult {
        assert!(self.num_matched() == 1);

        self.matched_vec.get_mut()[0].1.clone()
    }

    // Return true if candidate is executable.
    fn is_candidate_executable(&mut self) -> bool {
        assert!(self.num_matched() == 1);
        
        self.matched_vec.get_mut()[0].0.is_executable()
    }

    // Fill matched vec with next nodes from current node.
    fn set_matched_vec(&mut self, node: Rc<CliNode>) {
        self.matched_vec.replace(
            node.inner().next().iter()
                .map(|n| (n.clone(), MatchResult::Success(MatchFlag::Partial)))
                .collect());
    }

    // Remove hidden node from matched.
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
    pub fn token_len(&self) -> usize {
        self.token.len()
    }

    // Take node token_vec.
    pub fn node_token_vec(&self) -> CliNodeTokenVec {
        self.node_token_vec.take()
    }

    // Select matched nodes with MatchFlag smaller than or equal to 'limit'.
    // Among matched nodes with the same MatchFlag and the smallest MatchFlag.
    fn filter_matched(&mut self, limit: MatchFlag) {
        let mut limit = limit;
        let mut vec = self.matched_vec.replace(Vec::new());

        loop {
            if let Some(n) = vec.pop() {
                match n.1 {
                    MatchResult::Success(flag) => {
                        if flag > limit {
                            continue;
                        }
                        else if flag < limit {
                            self.matched_vec.get_mut().clear();
                            limit = flag;
                        }

                        self.matched_vec.get_mut().push(n);
                    },
                    _ => {}
                }
            }
            else {
                break;
            }
        }
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

    // Try match shorter string in line.
    fn match_shorter(&mut self, curr: Rc<CliNode>, token: String) {
        let mut len = token.len();

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

        self.matched_len = self.current_pos() - token.len() + len;
    }

    // Parse line and match current node for completion.
    pub fn parse(&mut self, curr: Rc<CliNode>) -> ExecResult {
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

                    return ExecResult::Unrecognized(self.matched_len)
                }

                // Matched more than one, ambiguous.
                if self.num_matched() > 1 {
                    self.filter_matched(MatchFlag::Full);
                    if self.num_matched() == 1 {
                        let next = self.get_candidate();
                        return self.parse(next);
                    }

                    return ExecResult::Ambiguous
                }
                // Matched one, move to next node.
                else {
                    let next = self.get_candidate();
                    return self.parse(next);
                }
            }

            // Not yet at the end of input, but no match.
            if self.num_matched() == 0 {
                self.match_shorter(curr.clone(), token.clone());
                if curr.inner().next().len() == 0 {
                    self.matched_len += 1;
                }

                return ExecResult::Unrecognized(self.matched_len)
            }
            else if self.num_matched() == 1 {
                curr = self.get_candidate();
            }

            break;
        }

        self.matched_len = self.current_pos();
        self.executable = curr.inner().is_executable();
        if self.executable {
            ExecResult::Complete
        }
        else {
            ExecResult::Incomplete
        }
    }

    // Parse line and match current node for execution.
    pub fn parse_execute(&mut self, curr: Rc<CliNode>) -> ExecResult {
        let mut executable = curr.is_executable();

        loop {
            if !self.trim_start() {
                break;
            }

            if curr.inner().next().len() == 0 {
                if self.line().trim().len() == 0 {
                    break;
                }

                self.matched_len = self.current_pos();
                return ExecResult::Unrecognized(self.matched_len)
            }

            self.set_matched_vec(curr.clone());

            let token = match self.get_token() {
                Some(token) => token,
                None => break,
            };

            self.save_token(&token);
            self.match_token(&token, curr.clone());
            self.filter_matched(MatchFlag::Partial);

            if self.num_matched() == 0 {
                self.match_shorter(curr.clone(), token.clone());
                return ExecResult::Unrecognized(self.matched_len)
            }
            else if self.num_matched() > 1 {
                return ExecResult::Ambiguous
            }

            // Candidate is only one at this point.
            let tuple: CliNodeTokenTuple = (self.get_candidate(), token.clone());
            self.node_token_vec.get_mut().push(tuple);

            // Line is still remaining, move forward.
            if !self.line().is_empty() {
                let next = self.get_candidate();
                return self.parse_execute(next)
            }

            // No more line, make decision.
            if self.get_candidate_result() == MatchResult::Success(MatchFlag::Incomplete) {
                return ExecResult::Incomplete
            }

            if !self.is_candidate_executable() {
                return ExecResult::Incomplete
            }
            else {
                return ExecResult::Complete
            }
//            break;
        }

        if executable {
            ExecResult::Complete
        }
        else {
            ExecResult::Incomplete
        }
    }
}


#[cfg(test)]
mod tests {
    use super::super::tree::*;
    use super::*;

    #[test]
    pub fn test_get_token() {
        let mut p = CliParser::new();
        p.init("show ip ospf interface");

        let ret = p.trim_start();
        assert_eq!(ret, true);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(p.line(), String::from(" ip ospf interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(p.line(), String::from(" ospf interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(p.line(), String::from(" interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("interface")));
        assert_eq!(p.line(), String::from(""));
    }

    #[test]
    pub fn test_get_token_space() {
        let mut p = CliParser::new();
        p.init(" show   ip ospf ");

        let ret = p.trim_start();
        assert_eq!(ret, true);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(p.line(), String::from("   ip ospf "));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(p.line(), String::from(" ospf "));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(p.line(), String::from(" "));

        let _ret = p.trim_start();
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


        let mut p = CliParser::new();
        p.init("show ip ospf");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Incomplete);

        p.init("show x");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Unrecognized(5));

        p.init("show ip xy");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Unrecognized(8));

        p.init("s i o i");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Ambiguous);

        p.init("s ip o i");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Complete);

        p.init("s ipv o i");
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Complete);
    }
}
