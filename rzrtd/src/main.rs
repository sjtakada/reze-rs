//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//

use log::info;
use simplelog::*;

use rzrtd::core::nexus::RouterNexus;

fn main() {
    // TODO: command line arguments.

    // Init logger
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, Config::default()).unwrap()
        ]
    ).unwrap();

    // Start daemon
    info!("ReZe Router Daemon started.");

    let mut nexus = RouterNexus::new();
    nexus.start();

    info!("ReZe RouterD terminated.");
}
