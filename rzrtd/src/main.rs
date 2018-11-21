//
// ReZe.Rs - Router Daemon
// Copyright (C) 2018 Toshiaki Takada
//

extern crate rzrtd;
extern crate tokio;
extern crate futures;

use std::sync::Arc;
use tokio::runtime::current_thread;

use rzrtd::core::bootstrap::BootStrap;

fn main() {
    // TODO: command line arguments.

    // BootStrap.
    println!("ReZe RouterD started.");

    let mut bootstrap = BootStrap::new();
    bootstrap.run();

    println!("ReZe RouterD terminated.");
}
