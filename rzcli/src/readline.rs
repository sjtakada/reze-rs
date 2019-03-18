//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline, rustyline integration.
//

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::line_buffer::LineBuffer;
use rustyline::error::ReadlineError;
use rustyline::Helper;
use rustyline::Editor;
use rustyline::KeyPress;

use super::tree::CliTree;

pub struct CliCompleter<'a> {
    trees: &'a HashMap<String, Rc<CliTree>>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            trees: trees
        }
    }

    /*
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
    */
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

