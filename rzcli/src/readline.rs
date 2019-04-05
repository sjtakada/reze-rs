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
use rustyline::Context;
use rustyline::KeyPress;
use rustyline::config;

use super::tree::CliTree;
use super::parser::*;

const CLI_INITIAL_MODE: &str = "CONFIG-MODE";

pub struct CliCompleter<'a> {
    // Reference to CLI command tree map.
    trees: &'a HashMap<String, Rc<CliTree>>,

    // Current tree.
    current: Rc<CliTree>,

    // CLI parser.
    parser: RefCell<CliParser>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>, mode: &str) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            trees: trees,
            current: trees[mode].clone(),
            parser: RefCell::new(CliParser::new()),
        }
    }
}

impl<'a> Completer for CliCompleter<'a> {
    type Candidate = String;

    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<String>)> {
        let mut candidate: Vec<String> = Vec::new();
        let mut parser = self.parser.borrow_mut();
        let line = line.trim_start();

        parser.set_line(&line);
        parser.parse(self.current.top());

        let vec = parser.matched_vec(); 
        for n in vec {
            candidate.push(n.0.inner().token().to_string());
        }

        Ok((parser.current_pos() - parser.token_len(), candidate))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }

    fn custom(&self, line: &str, _pos: usize, _ctx: &Context<'_>, _c: char) -> rustyline::Result<()> {
        let line = line.trim_start();
        let mut parser = self.parser.borrow_mut();

        parser.set_line(&line);
        parser.parse(self.current.top());

        let vec = parser.matched_vec(); 
        if vec.len() > 0 {
            if let Some(max) = vec.iter().map(|n| n.0.inner().token().len()).max() {
                for n in vec {
                    println!("  {:width$}  {}", n.0.inner().token(), n.0.inner().help(), width = max);
                }
            }

            println!("");
        }
        else {
            println!("% Unrecognized command");
        }

        Ok(())
    }
}

impl<'a> Hinter for CliCompleter<'a> {}


pub struct CliReadline<'a> {
    // CLI mode to tree map.
    trees: &'a HashMap<String, Rc<CliTree>>,

    // CLI completer.
    editor: RefCell<Editor<CliCompleter<'a>>>,

    // Current CLI mode.
    mode: Cell<String>,
}

impl<'a> CliReadline<'a> {
    pub fn new(trees: &'a HashMap<String, Rc<CliTree>>) -> CliReadline<'a> {
        // Set configuration.
        let config = config::Builder::new()
            .completion_type(config::CompletionType::List)
            .build();

        let mut editor = Editor::<CliCompleter>::with_config(config);
        editor.set_helper(Some(CliCompleter::new(trees, CLI_INITIAL_MODE)));

        // Bind '?' as hint.
        editor.bind_sequence(KeyPress::Char('?'), Cmd::Custom('?'));


        CliReadline::<'a> {
            trees: trees,
            editor: RefCell::new(editor),
            mode: Cell::new(String::from(CLI_INITIAL_MODE)),
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

