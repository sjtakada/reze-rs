//
// ReZe.Rs - Router Daemon
// Copyright (C) 2018 Toshiaki Takada
//

extern crate rzrtd;
extern crate tokio;
extern crate futures;

use tokio::runtime::current_thread;

use rzrtd::core::bootstrap::BootStrap;

fn main() {
    let bootstrap = BootStrap::new();

    // Start bootstrap thread in this context.
    let mut runtime = current_thread::Runtime::new().unwrap();
    runtime.spawn(bootstrap);
    runtime.run().unwrap();

    println!("ReZe RouterD terminated.");
}

