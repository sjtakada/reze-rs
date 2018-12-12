//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//

use log::info;
use simplelog::*;

use rzrtd::core::master::RouterMaster;

fn main() {
    // TODO: command line arguments.

    // Init logger
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default()).unwrap()
        ]
    ).unwrap();

    // Start daemon
    info!("ReZe RouterD started.");

    let mut master = RouterMaster::new();
    master.start();

    info!("ReZe RouterD terminated.");
}
