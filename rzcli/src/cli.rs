//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Main
//

//use std::io::*;

use std::env;
use mio_uds::UnixStream;

use std::cell::RefCell;

use super::error::*;
use super::readline::*;

pub struct Cli {
    readline: RefCell<CliReadline>,
}

impl Cli {
    pub fn new() -> Cli {
        Cli {
            readline: RefCell::new(CliReadline::new()),
        }
    }

    pub fn init(&self) -> Result<(), CliError> {
        let mut path = env::temp_dir();
        path.push("rzrtd.cli");

        let mut stream = match UnixStream::connect(path) {
            Ok(mut stream) => stream,
            Err(_) => return Err(CliError::ConnectError),
        };
        
        Ok(())
    }

    pub fn run(&self) {
        loop {
            // TODO, we'll get API URL and parameters here to send to server.
            self.readline.borrow_mut().gets();

            /*
            stdout().write(b"> ");
            stdout().flush();

            let mut buffer = String::new();
            stdin().read_line(&mut buffer);

            stream.write(buffer.as_ref());
            stream.flush();
             */
        }
    }
}
