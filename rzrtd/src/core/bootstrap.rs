//
// ReZe.Rs - Router Daemon
//           BootStrap to initialize basic event handlers.
//
// Copyright (C) 2018 Toshiaki Takada
//

use std::io;
use std::io::prelude::*;
//use std::sync::Arc;
//use tokio::runtime;
use tokio::runtime::current_thread;
use futures::{Future, Async, Poll};

pub struct BootStrap {

}

impl BootStrap {
    pub fn new() -> BootStrap {
        BootStrap { }
    }

    pub fn run(&mut self) {
        let mut runtime = match current_thread::Runtime::new() {
            Ok(runtime) => runtime,
            Err(err) => panic!("Error: {}", err)
        };

        let task = BootStrapTask { };

        runtime.spawn(task);
        runtime.run();
    }
}

struct BootStrapTask {

}

impl Future for BootStrapTask {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut line).unwrap();

            match line.as_ref() {
                "ospf\n" => {
                    println!("% start ospf {}", line);
                },
                "end\n" => {
                    println!("% end");
                    break;
                }
                _ => {
                    println!("% Unknown command");
                }
            }
        }

        Ok(Async::Ready(()))
    }
}


