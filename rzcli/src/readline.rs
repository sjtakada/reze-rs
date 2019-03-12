//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline
//

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



pub struct CliCompleter<'a> {
    trees: &'a HashMap<String, Rc<CliTree>>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            trees: trees
        }
    }
}

impl<'a> Completer for CliCompleter<'a> {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        let mut candidate: Vec<String> = Vec::new();

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
