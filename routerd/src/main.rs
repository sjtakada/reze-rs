//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//

use std::env;
use std::sync::Arc;

use log::info;
use simplelog::*;

use routerd::core::event::*;
use routerd::core::nexus::*;
use routerd::core::uds_server::*;

// Global entry point of ReZe Router Daemon.
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

    start();

    info!("ReZe Router Daemon terminated.");
}

// Initialize objects and associate them.
// TODO: probably take config or command line parameters.
fn start() {
    // Create Unix Domain Socket to accept commands.
    let mut path = env::temp_dir();
    path.push("routerd.cli");

    // Prepare some objects.
    let event_manager = Arc::new(EventManager::new());
    let nexus = Arc::new(RouterNexus::new());
    let _uds_server = UdsServer::start(event_manager.clone(), nexus.clone(), &path);

    // Start nexus.
    nexus.start(event_manager);
}

