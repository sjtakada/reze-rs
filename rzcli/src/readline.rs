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
use super::parser::*;

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
        let line = line.trim_start();

        // TBD: where am I?   should keep which mode I am.
        let tree = &self.trees["VIEW-NODE"];

        let mut parser = CliParser::new(&line);
        parser.parse(tree.top());

/*
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
*/

        println!("");
        let vec = parser.matched_vec(); 
        for n in vec {
            println!("{}", n.0.inner().token());
            candidate.push(n.0.inner().token().to_string());
        }

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

