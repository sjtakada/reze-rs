//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Readline
//

use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::line_buffer::LineBuffer;
use rustyline::error::ReadlineError;
//use rustyline::Helper;
use rustyline::Editor;

pub struct CliCompleter {
}

impl Completer for CliCompleter {
    type Candidate = String;

    fn complete(&self, line: &str, pos: usize)
                -> rustyline::Result<(usize, Vec<String>)> {
        let candidate: Vec<String> = Vec::new();

        Ok((0usize, candidate))
    }

    fn update(&self, _line: &mut LineBuffer, _start: usize, _elected: &str) {

    }
}

impl Hinter for CliReadline {
    fn hint(&self, _line: &str, _pos: usize) -> Option<String> {
        None
    }
}


pub struct CliReadline {
    // Parent CLI object.

    // Readline buffer.
    buf: [u8; 1024],

    // Completion matched string vector.
    //matched_strvec: Vec<&str>,
    matched_index: usize,
}

impl CliReadline {
    pub fn new() -> CliReadline{
        CliReadline {
            buf: [0; 1024],
            matched_index: 0
        }
    }

    // Setup Readline.
    pub fn init(&self) {

    }

    pub fn gets(&self) {
        let mut rl = Editor::<()>::new();

        loop {
            let readline = rl.readline("Router>");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    println!("Line: {}", line);
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break
                },
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                    break
                }
            }
        }
    }
}

//impl Highlighter for CliReadline {}
//impl Helper for CliReadline {}
