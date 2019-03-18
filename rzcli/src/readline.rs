//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline
//

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use serde_json;

use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::line_buffer::LineBuffer;
use rustyline::error::ReadlineError;
use rustyline::Helper;
use rustyline::Editor;
use rustyline::KeyPress;

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

pub struct CliParseState {
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

impl CliParseState {
    pub fn new(buf: &str) -> CliParseState {
        let input = String::from(buf);
        let mut line = String::from(" ");
        line.push_str(buf);

        CliParseState {
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
        assert!(self.num_matched() != 1);

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
}


pub struct CliCompleter<'a> {
    trees: &'a HashMap<String, Rc<CliTree>>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            trees: trees
        }
    }

    fn parse(&self, ps: &mut CliParseState, node: Rc<CliNode>) -> CliExecResult {
        let mut curr = node;

        loop {
            ps.matched_vec.get_mut().clear();  // TBD: is it needed?
            if !ps.trim_start() {
                break;
            }

            ps.set_matched_vec(curr.clone());
            ps.filter_hidden();

            let token = match ps.get_token() {
                Some(token) => token,
                None => break,
            };

            ps.save_token(&token);
            ps.match_token(&token, curr.clone());

            // At the end of input.
            if !ps.line.is_empty()  {
                ps.filter_matched(MatchFlag::Partial);

                // No match, try shorter to find one.
                if ps.num_matched() == 0 {
                    ps.match_shorter(curr.clone(), token.clone());
                    if curr.inner().next().len() == 0 {
                        ps.matched_len_inc();
                    }

                    return CliExecResult::Unrecognized
                }
                // Matched more than one, ambiguous.
                else if ps.num_matched() > 1 {
                    return CliExecResult::Ambiguous
                }
                // Matched one, move to next node.
                else {
                    let next = ps.get_next();
                    return self.parse(ps, next);
                }
            }

            // Not yet at the end of input, but no match.
            if ps.num_matched() == 0 {
                ps.match_shorter(curr.clone(), token.clone());
                if curr.inner().next().len() == 0 {
                    ps.matched_len_inc();
                }

                return CliExecResult::Unrecognized
            }
            else if ps.num_matched() == 1 {
                curr = ps.get_next();
            }

            break;
        }

        ps.is_cmd = curr.inner().is_cmd();
        if ps.is_cmd {
            CliExecResult::Complete
        }
        else {
            CliExecResult::Incomplete
        }
    }
}

impl<'a> Completer for CliCompleter<'a> {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let mut candidate: Vec<String> = Vec::new();
        let line = line.trim_start();

        match line.chars().next() {
            Some(c) => {
                match c {
                    'u' => candidate.push("udon".to_string()),
                    'r' => candidate.push("ramen".to_string()),
                    's' => candidate.push("soba".to_string()),
                    _ => {}
                }
            },
            None => {
            }
        }

//        println!("");
//        for i in candidate.iter() {
//            println!("{}", i);
//        }

        Ok((0, candidate))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }
}

impl<'a> Hinter for CliCompleter<'a> {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
//        Some("hoge".to_string())
        None
    }
}


pub struct CliReadline<'a> {
    // CLI mode to tree map.
    trees: &'a HashMap<String, Rc<CliTree>>,

    // CLI completer.
    editor: RefCell<Editor<CliCompleter<'a>>>,

    // Readline buffer.
    //buf: [u8; 1024],

    // Completion matched string vector.
    //matched_strvec: Vec<&str>,
    matched_index: usize,
}

impl<'a> CliReadline<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliReadline<'a> {
        let mut editor = Editor::<CliCompleter>::new();
        editor.set_helper(Some(CliCompleter::new(trees)));

        CliReadline::<'a> {
            trees: trees,
            editor: RefCell::new(editor),
            matched_index: 0,
        }
    }

    pub fn gets(&self) -> Result<String, ReadlineError> {
        let mut editor = self.editor.borrow_mut();

        let readline = editor.readline("Router>");
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_ref());
                println!("Line: {}", line);
                Ok(line)
            },
            Err(err) => Err(err)
        }
    }
}

impl<'a> Highlighter for CliCompleter<'a> {}
impl<'a> Helper for CliCompleter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_token() {
        let mut ps = CliParseState::new("show ip ospf interface");

        let ret = ps.trim_start();
        assert_eq!(ret, true);

        let ret = ps.trim_start();
        assert_eq!(ret, false);

        let token = ps.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(ps.line(), String::from(" ip ospf interface"));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(ps.line(), String::from(" ospf interface"));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(ps.line(), String::from(" interface"));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, Some(String::from("interface")));
        assert_eq!(ps.line(), String::from(""));
    }

    #[test]
    pub fn test_get_token_space() {
        let mut ps = CliParseState::new(" show   ip ospf ");

        let ret = ps.trim_start();
        assert_eq!(ret, true);

        let ret = ps.trim_start();
        assert_eq!(ret, false);

        let token = ps.get_token();
        assert_eq!(token, Some(String::from("show")));
        assert_eq!(ps.line(), String::from("   ip ospf "));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, Some(String::from("ip")));
        assert_eq!(ps.line(), String::from(" ospf "));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, Some(String::from("ospf")));
        assert_eq!(ps.line(), String::from(" "));

        let ret = ps.trim_start();
        let token = ps.get_token();
        assert_eq!(token, None);
        assert_eq!(ps.line(), String::from(""));
    }
}
