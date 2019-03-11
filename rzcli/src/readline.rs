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



pub struct CliCompleter {
//    trees: Rc<HashMap<String, Rc<CliTree>>>,
}

impl CliCompleter {
    pub fn new(trees: Rc<HashMap<String, Rc<CliTree>>>) -> CliCompleter {
        CliCompleter {
//            trees: trees
        }
    }
}

impl Completer for CliCompleter {
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

impl Hinter for CliCompleter {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
//        Some("hoge".to_string())
        None
    }
}


pub struct CliReadline {
    // CLI mode to tree map.
    trees: RefCell<HashMap<String, Rc<CliTree>>>,

    // CLI completer.
    editor: RefCell<Editor<CliCompleter>>,

    // Readline buffer.
    //buf: [u8; 1024],

    // Completion matched string vector.
    //matched_strvec: Vec<&str>,
    matched_index: usize,
}

impl CliReadline {
    pub fn new() -> CliReadline {
        let mut editor = Editor::<CliCompleter>::new();
        editor.set_helper(Some(CliCompleter { }));

        CliReadline {
            trees: RefCell::new(HashMap::new()),
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

    // Parse single 'defun' definition.
    pub fn parse_defun(&mut self, tokens: &serde_json::Value,
                       command: &serde_json::Value) {
        if command["mode"].is_array() {
            for mode in command["mode"].as_array().unwrap() {
                if let Some(mode) = mode.as_str() {
                    if let Some(tree) = self.trees.borrow_mut().get(mode) {
                        tree.build_command(tokens, command);
                    }
                }
            }
        }
    }

    // Build CLI mode tree from JSON.
    fn build_mode(&mut self, json: &serde_json::Value, parent: Option<Rc<CliTree>>) -> Result<(), CliError> {
        for name in json.as_object().unwrap().keys() {
            let mode = &json[name];
            if mode.is_object() {
                let prompt = if mode["prompt"].is_string() {
                    &mode["prompt"].as_str().unwrap()
                } else {
                    ">"
                };
                let children = &mode["children"];
                let tree = Rc::new(CliTree::new(name.to_string(), prompt.to_string(), parent.clone()));
                self.trees.borrow_mut().insert(name.to_string(), tree.clone());

                if children.is_object() {
                    self.build_mode(&children, Some(tree.clone()));
                }
            }
        }

        Ok(())
    }
}

impl Highlighter for CliCompleter {}
impl Helper for CliCompleter {}
