//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline, rustyline integration.
//

use std::collections::HashMap;
use std::cell::RefCell;
//use std::cell::Cell;
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
use super::node::NodeType;
//use super::node::Value;
use super::error::CliError;

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

        parser.init(&line, self.cli.privilege());
        parser.parse(current.top());

        let vec = parser.matched_vec(); 
        if vec.len() == 1 {
            let node = &vec[0].0;
            if node.node_type() == NodeType::Keyword {
                let mut str = node.inner().display().to_string();
                str.push(' ');

                candidate.push(str);
            }
        }
        else {
            for n in vec {
                let node = &n.0;
                if node.node_type() == NodeType::Keyword {
                    candidate.push(node.inner().display().to_string());
                }
            }
        }

        Ok((parser.pos_prev(), candidate))
    }

    fn update(&self, line: &mut LineBuffer, start: usize, elected: &str) {
        let end = line.pos();
        line.replace(start..end, elected)
    }

    fn custom(&self, line: &str, _pos: usize, _ctx: &Context<'_>, _c: char) -> rustyline::Result<()> {
        let line = line.trim_start();
        let mut parser = self.parser.borrow_mut();
        let current = self.cli.current().unwrap();

        parser.init(&line, self.cli.privilege());
        let result = parser.parse(current.top());
        match result {
            ExecResult::Unrecognized(_pos) => {
                println!("% Unrecognized command");
            },
            _ => {

                let vec = parser.matched_vec(); 
                let mut width_max = 0;
                if vec.len() > 0 {
                    if let Some(max) = vec.iter().map(|n| n.0.inner().display().len()).max() {
                        width_max = max;
                        for n in vec {
                            println!("  {:width$}  {}", n.0.inner().display(), n.0.inner().help(), width = max);
                        }
                    }
                }

                if result == ExecResult::Complete {
                    println!("  {:width$}  <cr>", "<cr>", width = width_max);
                }
                println!("");
            }
        }

        Ok(())
    }

    fn short_help(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> rustyline::Result<Vec<String>> {
        let mut candidate: Vec<String> = Vec::new();
        let mut parser = self.parser.borrow_mut();
        let line = line.trim_start();
        let current = self.cli.current().unwrap();

        parser.init(&line, self.cli.privilege());
        parser.parse(current.top());

        let vec = parser.matched_vec(); 
        if vec.len() == 1 {
            let node = &vec[0].0;
            let mut str = node.inner().display().to_string();
            str.push(' ');
            candidate.push(str);
        }
        else {
            for n in vec {
                let node = &n.0;
                candidate.push(node.inner().display().to_string());
            }
        }

        Ok(candidate)
    }

/*
    fn short_help(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> rustyline::Result<()> {
        let line = line.trim_start();
        let mut parser = self.parser.borrow_mut();
        let current = self.cli.current().unwrap();

        parser.init(&line, self.cli.privilege());
        let result = parser.parse(current.top());
        match result {
            ExecResult::Unrecognized(pos) => {
                println!("% Unrecognized command");
            },
            _ => {

                let vec = parser.matched_vec(); 
                let mut width_max = 0;
                if vec.len() > 0 {
                    if let Some(max) = vec.iter().map(|n| n.0.inner().token().len()).max() {
                        width_max = max;
                        for n in vec {
                            println!("  {:width$}  {}", n.0.inner().token(), n.0.inner().help(), width = max);
                        }
                    }
                }

                if result == ExecResult::Complete {
                    println!("  {:width$}  <cr>", "<cr>", width = width_max);
                }
                println!("");
            }
        }

        Ok(())
    }
*/
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
            .allow_suspend(false)
            .short_help(true)
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
        let readline = editor.readline(&self.cli.prompt());
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

            parser.init(&line, self.cli.privilege());

            let mut result = parser.parse_execute(current.top());
            match result {
                ExecResult::Complete => {
                    match self.handle_actions(&mut parser) {
                        Err(CliError::NoActionDefined) => {
                            println!("% No action defined");
                        },
                        Err(CliError::ActionError(msg)) => {
                            println!("% Action error: {}", msg);
                        },
                        Err(_) => {
                            println!("% Unknown action error");
                        },
                        Ok(()) => {
                            //println!("execute {}", line);
                        },
                    }
                },
                ExecResult::Incomplete => {
                    println!("% Incomplete command");
                },
                ExecResult::Ambiguous => {
                    println!("% Ambiguous command");
                },
                ExecResult::Unrecognized(pos) => {
                    let pos = pos + self.cli.prompt().len();

                    if let Some(parent) = current.parent() {
                        result = self.execute_parent(&mut parser, parent);
                    }

                    if result != ExecResult::Complete {
                        println!("{:>width$}^", "", width = pos);
                        println!("% Invalid input detected at '^' marker.");
                    }
                },
            }
        }
    }

    fn execute_parent(&self, parser: &mut CliParser, current: Rc<CliTree>) -> ExecResult {
        parser.reset_line();

        let mut result = parser.parse_execute(current.top());
        match result {
            ExecResult::Complete => {
                match self.handle_actions(parser) {
                    Err(CliError::NoActionDefined) => {
                        println!("% No action defined");
                    }
                    Err(_) => {
                        println!("% Unknown action error in parent node");
                    }
                    Ok(()) => {
                        self.cli.set_mode(current.name()).unwrap();
                        //println!("execute {}", line);
                    }
                }

                // We set mode up, if the command exists.
                self.cli.set_mode(current.name()).unwrap();
            }
            _ => {
                match current.parent() {
                    Some(parent) => {
                        result = self.execute_parent(parser, parent);
                    },
                    None => {
                        // follow through
                    }
                }
            }
        }

        result
    }

    fn handle_actions(&self, parser: &mut CliParser) -> Result<(), CliError> {
        // Populate mode params first.
        println!("** handle_actions");

/*
        // Populate params and keywords.
        for n in node_token_vec.iter() {
            let node = n.0.clone();

            if self.cli.is_debug() {

            }
        }

*/
        let node = parser.node_executable().unwrap();

        if node.inner().actions().len() > 0 {
            for action in node.inner().actions().iter() {
                action.handle(&self.cli)?;
            }
            Ok(())
        }
        else {
            Err(CliError::NoActionDefined)
        }
    }
}

impl<'a> Highlighter for CliCompleter<'a> {}
impl<'a> Helper for CliCompleter<'a> {}

