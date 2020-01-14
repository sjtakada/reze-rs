//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Parser.
//

use std::fmt;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use std::collections::HashMap;

//use serde_json;

//use super::error::CliError;
use super::node::CliNode;
use super::node::Value;
use super::collate::*;

// Constants.
const CLI_DEFAULT_PARSER_PRIVILEGE: u8 = 1;

// Type aliases.
type CliNodeMatchStateTuple = (Rc<dyn CliNode>, MatchResult);
type CliNodeMatchStateVec = Vec<CliNodeMatchStateTuple>;

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

    // Current position.
    pos: Cell<usize>,

    // Previous position.
    pos_prev: Cell<usize>,

    // Matched length.
    matched_len: Cell<usize>,

    // Vector of pair of CliNode and MatchState.
    matched_vec: RefCell<CliNodeMatchStateVec>,

    // Last matched node for execution.
    node_executable: Option<Rc<dyn CliNode>>,

    // HashMap of captured key value pairs.
    captured_map: RefCell<HashMap<String, Value>>,

    // Record id of node with only once flag.
    only_once_set: RefCell<HashSet<String>>,

    // Current privilege level.
    privilege: u8,

    // Return value, whether or not it hits executable command.
    executable: bool,
}

impl CliParser {
    // Constructor.
    pub fn new() -> CliParser {
        CliParser {
            input: String::new(),
            pos: Cell::new(0usize),
            pos_prev: Cell::new(0usize),
            matched_len: Cell::new(0usize),
            matched_vec: RefCell::new(Vec::new()),
            node_executable: None,
            captured_map: RefCell::new(HashMap::new()),
            only_once_set: RefCell::new(HashSet::new()),
            privilege: CLI_DEFAULT_PARSER_PRIVILEGE,
            executable: false,
        }
    }

    // Set input and reset state.
    pub fn init(&mut self, input: &str, privilege: u8) {
        self.input = String::from(input);
        self.privilege = privilege;
        self.reset_line();
    }

    // Reset line and other parser state.
    pub fn reset_line(&mut self) {
        self.pos.set(0);
        self.matched_len.set(0);
        self.matched_vec.replace(Vec::new());
        self.node_executable = None;
        self.captured_map.replace(HashMap::new());
        self.only_once_set.borrow_mut().clear();
        self.executable = false;
    }

    // Return parser cursor position.
    pub fn pos(&self) -> usize {
        self.pos.get()
    }

    // Return previouls position.
    pub fn pos_prev(&self) -> usize {
        self.pos_prev.get()
    }

    // Return remaining input length.
    pub fn line_len(&self) -> usize {
        self.line().len()
    }

    // Return current matched vec and set it empty.
    pub fn matched_vec(&self) -> CliNodeMatchStateVec {
        self.matched_vec.replace(Vec::new())
    }

    // Return current matched string len.
    pub fn matched_len(&self) -> usize {
        self.matched_len.get()
    }

    // Return reference to current remaining line string.
    pub fn line(&self) -> &str {
        &self.input[self.pos.get()..]
    }

    // Return executable node.
    pub fn node_executable(&self) -> Option<Rc<dyn CliNode>> {
        self.node_executable.clone()
    }

    // Return number of matched in current matched.
    fn num_matched(&self) -> usize {
        self.matched_vec.borrow_mut().len()
    }

    // Return candidate in matched.
    fn get_candidate(&self) -> Rc<dyn CliNode> {
        assert!(self.num_matched() == 1);

        self.matched_vec.borrow_mut()[0].0.clone()
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
    fn set_matched_vec(&self, node: Rc<dyn CliNode>) {
        self.matched_vec.replace(
            node.inner().next().iter()
                .map(|n| (n.clone(), MatchResult::Success(MatchFlag::Partial)))
                .collect());
    }

    // Remove hidden node from matched.
    fn filter_hidden(&self) {
        self.matched_vec.borrow_mut().retain(|n| !n.0.inner().is_hidden());
    }

    // Remove the node already appeared with only once flag.
    fn filter_only_once(&self) {
        let mut matched_vec = Vec::new();

        // TODO: refactoring
        for n in self.matched_vec.borrow_mut().iter() {
            if !self.only_once_set.borrow().contains(n.0.inner().id()) {
                matched_vec.push((n.0.clone(), n.1));
            }
        }

        self.matched_vec.replace(matched_vec);
    }

    // Remove node with prvilege greater than current privilege level.
    fn filter_privilege(&self) {
        let privilege = self.privilege;
        self.matched_vec.borrow_mut().retain(|n| n.0.inner().privilege() <= privilege);
    }

    // Return true if space exists at the beginning and trim it, or return false.
    fn trim_start(&mut self) -> bool {
        let s = self.line().trim_start();
        let offset = self.line_len() - s.len();

        if offset > 0 {
            self.pos.set(self.pos.get() + offset);
            true
        }
        else {
            false
        }
    }

    // Return string slice of an input token from current position.
    fn get_token(&self) -> Option<&str> {
        self.pos_prev.set(self.pos());

        if self.line_len() == 0 {
            None
        }
        else {
            let pos = match self.line().find(|c: char| c.is_whitespace()) {
                Some(pos) => pos,
                None => self.line_len(),
            };

            self.pos.set(self.pos.get() + pos);
            Some(&self.input[self.pos_prev.get()..self.pos.get()])
        }
    }

    // Select matched nodes with MatchFlag smaller than or equal to 'limit'.
    // Among matched nodes with the same MatchFlag and the smallest MatchFlag.
    fn filter_matched(&self, limit: MatchFlag) {
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
                            self.matched_vec.borrow_mut().clear();
                            limit = flag;
                        }

                        self.matched_vec.borrow_mut().push(n);
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
    fn match_token(&self, token: &str, curr: Rc<dyn CliNode>) {
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
    fn match_shorter(&self, token: &str, curr: Rc<dyn CliNode>) {
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

        self.matched_len.set(self.pos() - token.len() + len);
    }

    // Parse line and match current node for completion.
    pub fn parse(&mut self, curr: Rc<dyn CliNode>) -> ExecResult {
        let mut curr = curr;

        loop {
            self.trim_start();
            self.set_matched_vec(curr.clone());
            self.filter_hidden();
            self.filter_only_once();
            self.filter_privilege();

            if self.line_len() == 0 {
                break;
            }

            let token = match self.get_token() {
                Some(token) => token,
                None => break,
            };
            self.match_token(token, curr.clone());

            if curr.inner().is_only_once() {
                self.only_once_set.borrow_mut().insert(String::from(curr.inner().id()));
            }

            // Not yet at the end of input.
            if !self.line().is_empty()  {
                self.filter_matched(MatchFlag::Partial);

                // No match, try shorter to find one.
                if self.num_matched() == 0 {
                    self.match_shorter(token, curr.clone());
                    if curr.inner().next().len() == 0 {
                        self.matched_len.set(self.matched_len.get() + 1);
                    }

                    return ExecResult::Unrecognized(self.matched_len.get())
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

                    // Special case for LINE.
                    if next.is_line() {
                        self.matched_len.set(self.pos());
                        self.executable = next.inner().is_executable();
                        if self.executable {
                            return ExecResult::Complete
                        }
                        else {
                            return ExecResult::Incomplete
                        }
                    }

                    return self.parse(next);
                }
            }

            // Not yet at the end of input, but no match.
            if self.num_matched() == 0 {
                self.match_shorter(token, curr.clone());
                if curr.inner().next().len() == 0 {
                    self.matched_len.set(self.matched_len.get() + 1);
                }

                return ExecResult::Unrecognized(self.matched_len.get())
            }
            else if self.num_matched() == 1 {
                curr = self.get_candidate();
            }

            break;
        }

        self.matched_len.set(self.pos());
        self.executable = curr.inner().is_executable();
        if self.executable {
            ExecResult::Complete
        }
        else {
            ExecResult::Incomplete
        }
    }

    // Parse line and match current node for execution.
    pub fn parse_execute(&mut self, curr: Rc<dyn CliNode>) -> ExecResult {
        let executable = curr.is_executable();

        loop {
            self.trim_start();

            if curr.inner().next().len() == 0 {
                if self.line().trim().len() == 0 {
                    break;
                }

                self.matched_len.set(self.pos());
                return ExecResult::Unrecognized(self.matched_len.get())
            }

            self.set_matched_vec(curr.clone());

            let token = match self.get_token() {
                Some(token) => token,
                None => break,
            };

            self.filter_only_once();
            self.filter_privilege();

            self.match_token(token, curr.clone());
            self.filter_matched(MatchFlag::Partial);

            if self.num_matched() == 0 {
                self.match_shorter(token, curr.clone());
                return ExecResult::Unrecognized(self.matched_len.get())
            }
            else if self.num_matched() > 1 {
                return ExecResult::Ambiguous
            }

            // Candidate is only one at this point.
            let node = self.get_candidate();
            if node.is_line() {
                self.pos.set(self.pos_prev());
                node.capture(self.line(), &mut self.captured_map.borrow_mut());
            }
            else {
                node.capture(token, &mut self.captured_map.borrow_mut());
            }

            // Line is still remaining, move forward.
            if !self.line().is_empty() {
                // if next is LINE, will terminate.
                if !node.is_line() {
                    return self.parse_execute(node)
                }
            }

            self.node_executable = Some(node.clone());

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
        }

        if executable {
            self.node_executable = Some(curr.clone());
            ExecResult::Complete
        }
        else {
            ExecResult::Incomplete
        }
    }

    pub fn params_get(&self) -> HashMap<String, Value> {
        self.captured_map.replace(HashMap::new())
    }
}


#[cfg(test)]
mod tests {
    use super::super::tree::*;
    use super::*;

    const CLI_MAX_PARSER_PRIVILEGE: u8 = 15;

    #[test]
    pub fn test_get_token() {
        let mut p = CliParser::new();
        p.init("show ip ospf interface", CLI_MAX_PARSER_PRIVILEGE);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some("show"));
        assert_eq!(p.line(), String::from(" ip ospf interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some("ip"));
        assert_eq!(p.line(), String::from(" ospf interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some("ospf"));
        assert_eq!(p.line(), String::from(" interface"));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some("interface"));
        assert_eq!(p.line(), String::from(""));
    }

    #[test]
    pub fn test_get_token_space() {
        let mut p = CliParser::new();
        p.init(" show   ip ospf ", CLI_MAX_PARSER_PRIVILEGE);

        let ret = p.trim_start();
        assert_eq!(ret, true);

        let ret = p.trim_start();
        assert_eq!(ret, false);

        let token = p.get_token();
        assert_eq!(token, Some("show"));
        assert_eq!(p.line(), String::from("   ip ospf "));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some("ip"));
        assert_eq!(p.line(), String::from(" ospf "));

        let _ret = p.trim_start();
        let token = p.get_token();
        assert_eq!(token, Some("ospf"));
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
        p.init("show ip ospf", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Incomplete);

        p.init("show x", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Unrecognized(5));

        p.init("show ip xy", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Unrecognized(8));

        p.init("s i o i", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Ambiguous);

        p.init("s ip o i", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Complete);

        p.init("s ipv o i", CLI_MAX_PARSER_PRIVILEGE);
        let result = p.parse(tree.top());
        assert_eq!(result, ExecResult::Complete);
    }
}
