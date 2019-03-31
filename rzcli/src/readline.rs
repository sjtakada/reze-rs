//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline, rustyline integration.
//

use std::collections::HashMap;
use std::cell::RefCell;
use std::cell::Cell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::line_buffer::LineBuffer;
use rustyline::error::ReadlineError;
use rustyline::Cmd;
use rustyline::Helper;
use rustyline::Editor;
use rustyline::KeyPress;
use rustyline::config;

use super::tree::CliTree;
use super::parser::*;

pub struct CliCompleter<'a> {
    // Reference to CLI command tree map.
    trees: &'a HashMap<String, Rc<CliTree>>,

    // CLI parser.
    parser: RefCell<CliParser>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            trees: trees,
            parser: RefCell::new(CliParser::new()),
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
        let mut parser = self.parser.borrow_mut();

        parser.set_line(&line);
        parser.parse(tree.top());

//        println!("");
        let vec = parser.matched_vec(); 
        for n in vec {
//            println!("  {}", n.0.inner().token());
            candidate.push(n.0.inner().token().to_string());
        }

//println!("** matched_len {}", parser.matched_len());
        Ok((parser.current_pos() - parser.token_len(), candidate))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
//println!("*** update {}", start);

        let end = line.pos();
//        let offset = end - self.parser.borrow_mut().saved_token_size();

        line.replace(start..end, elected)
    }
}

impl<'a> Hinter for CliCompleter<'a> {
    fn hint(&self, line: &str, _pos: usize) -> Option<String> {
        if let Some(c) = line.chars().last() {
            if c != '?' {
                return None
            }

            let mut candidate: Vec<String> = Vec::new();
            let line = line.trim_start();

            // TBD: where am I?   should keep which mode I am.
            let tree = &self.trees["VIEW-NODE"];

            let mut parser = CliParser::new();
            parser.set_line(&line);
            parser.parse(tree.top());

            println!("");
            let vec = parser.matched_vec(); 
            for n in vec {
                println!("{}", n.0.inner().token());
            }

            return Some("".to_string())
        }

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
        // Set configuration.
        let mut config = config::Builder::new()
            .completion_type(config::CompletionType::List)
            .build();

        let mut editor = Editor::<CliCompleter>::with_config(config);
        editor.set_helper(Some(CliCompleter::new(trees)));

        // Bind '?' as hint.
        editor.bind_sequence(KeyPress::Char('?'), Cmd::CompleteHint);


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

