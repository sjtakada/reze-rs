//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//

use rzrtd::core::router_master::RouterMaster;

fn main() {
    // TODO: command line arguments.

    // Start daemon
    println!("ReZe RouterD started.");

    let mut master = RouterMaster::new();
    master.start();

    println!("ReZe RouterD terminated.");
}
