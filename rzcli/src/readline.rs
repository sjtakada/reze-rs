//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline
//

use std::cell::RefCell;

use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::line_buffer::LineBuffer;
use rustyline::error::ReadlineError;
use rustyline::Helper;
use rustyline::Editor;
use rustyline::KeyPress;

pub struct CliCompleter {
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
    // Parent CLI object.

    editor: RefCell<Editor<CliCompleter>>,

    // Readline buffer.
    //buf: [u8; 1024],

    // Completion matched string vector.
    //matched_strvec: Vec<&str>,
    matched_index: usize,
}

impl CliReadline {
    pub fn new() -> CliReadline{
        let mut editor = Editor::<CliCompleter>::new();
        editor.set_helper(Some(CliCompleter {}));

        CliReadline {
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

impl Highlighter for CliCompleter {}
impl Helper for CliCompleter {
}
