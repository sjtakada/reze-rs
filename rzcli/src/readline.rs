//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline, rustyline integration.
//

//use std::collections::HashMap;
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

use super::cli::Cli;
use super::tree::CliTree;
use super::parser::*;
use super::node::CliNode;

type CliNodeTokenTuple = (Rc<CliNode>, String);
type CliNodeTokenVec = Vec<CliNodeTokenTuple>;

//
pub struct CliCompleter<'a> {
    // Reference to CLI.
    cli: &'a Cli,

    // CLI parser.
    parser: RefCell<CliParser>,
}

impl<'a> CliCompleter<'a> {
    pub fn new(cli: &'a Cli) -> CliCompleter<'a> {
        CliCompleter::<'a> {
            cli: cli,
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
        let current = self.cli.current().unwrap();

        parser.init(&line);
        parser.parse(current.top());

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
        let current = self.cli.current().unwrap();

        parser.init(&line);
        parser.parse(current.top());

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

// CLI readline
//   Abstruction of rustyline.
//
pub struct CliReadline<'a> {
    // CLI
    cli: &'a Cli,

    // CLI completer.
    editor: RefCell<Editor<CliCompleter<'a>>>,
}

impl<'a> CliReadline<'a> {
    pub fn new(cli: &'a Cli) -> CliReadline<'a> {
        // Set configuration.
        let config = config::Builder::new()
            .completion_type(config::CompletionType::List)
            .build();

        let mut editor = Editor::<CliCompleter>::with_config(config);
        editor.set_helper(Some(CliCompleter::new(cli)));

        // Bind '?' as hint.
        editor.bind_sequence(KeyPress::Char('?'), Cmd::Custom('?'));


        CliReadline::<'a> {
            cli: cli,
            editor: RefCell::new(editor),
        }
    }

    pub fn gets(&self) -> Result<String, ReadlineError> {
        let mut editor = self.editor.borrow_mut();

        // TBD: prompt.
        let readline = editor.readline("Router>");
        match readline {
            Ok(line) => {
                editor.add_history_entry(line.as_ref());
                Ok(line)
            },
            Err(err) => Err(err)
        }
    }

    pub fn execute(&self, line: String) {
        if line.trim().len() > 0 {
            let mut parser = CliParser::new();
            let current = self.cli.current().unwrap();

            parser.init(&line);

            match parser.parse_execute(current.top()) {
                ExecResult::Complete => {
                    self.handle_actions(parser.node_token_vec());
                    println!("execute {}", line);
                },
                ExecResult::Incomplete => {
                    println!("% Incomplete command");
                },
                ExecResult::Ambiguous => {
                    println!("% Ambiguous command");
                },
                ExecResult::Unrecognized => {
                    println!("% Invalid input detected at");
                },
            }
        }
    }

    fn handle_actions(&self, node_token_vec: CliNodeTokenVec) {
        let (node, token) = node_token_vec.last().unwrap();

        for action in node.inner().actions().iter() {
            action.handle(&self.cli);
        }
    }
}

impl<'a> Highlighter for CliCompleter<'a> {}
impl<'a> Helper for CliCompleter<'a> {}

