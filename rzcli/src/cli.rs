//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Main
//

use std::env;
use std::io::*;
use mio_uds::UnixStream;

use super::readline;

pub struct Cli {
    
}

impl Cli {
    pub fn new() -> Cli {
        Cli { }
    }

    pub fn start(&self) {
        let mut path = env::temp_dir();
        path.push("rzrtd.cli");

        /*
        let mut stream = match UnixStream::connect(path) {
            Ok(mut stream) => stream,
            Err(_) => panic!("Error: cannot connect to Rzrtd")
        };
*/

        loop {
            let rl = readline::CliReadline::new();

            rl.gets();

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
