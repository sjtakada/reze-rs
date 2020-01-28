//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//

use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::fs;

use log::info;
use log::error;
use simplelog::*;
use getopts::Options;

use common::consts::*;
use common::error::*;
use common::event::*;
use common::uds_server::*;

use routerd::core::signal::*;
use routerd::core::nexus::*;
use routerd::core::mds::*;
use routerd::core::protocols::ProtocolType;

const ROUTERD_VERSION: &str = "0.1.0";

/// Help
fn print_help(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

/// Version.
fn print_version(program: &str) {
    println!("{} version {}", program, ROUTERD_VERSION);
    println!("{}", COPYRIGHT);
    println!("");
}

/// Global entry point of ReZe Router Daemon.
fn main() {
    // Command line arguments.
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("l", "loglevel", "Set log level (default debug)", "LOGLEVEL");
    opts.optflag("h", "help", "Display this help and exit");
    opts.optflag("v", "version", "Print program version");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(_err) => {
            println!("Invalid option");
            print_help(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_help(&program, opts);
        return;
    }

    if matches.opt_present("v") {
        print_version(&program);
        return;
    }

    let level_filter = if let Some(loglevel) = matches.opt_str("l") {
        match loglevel.as_ref() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Debug
        }
    } else {
        LevelFilter::Debug
    };

    // Init logger
    CombinedLogger::init(
        vec![
            TermLogger::new(level_filter, Config::default(), TerminalMode::Mixed).unwrap()
        ]
    ).unwrap();

    // Init Signals.
    signal_init();

    // Start daemon
    info!("ReZe Router Daemon started.");

    start();

    info!("ReZe Router Daemon terminated.");
}

// Initialize objects and associate them.
// TODO: probably take config or command line parameters.
fn start() {

    // Create Unix Domain Socket to accept configuration.
    let mut config_uds_path = env::temp_dir();
    config_uds_path.push(ROUTERD_CONFIG_UDS_FILENAME);

    // Prepare some objects.
    let event_manager = Arc::new(EventManager::new());
    let nexus = Arc::new(RouterNexus::new());

    // MdsHandlers.
    let zebra_handler = Rc::new(MdsProtocolHandler::new(ProtocolType::Zebra, nexus.clone()));

    let mds = Rc::new(MdsNode::new());
    MdsNode::register_handler(mds.clone(), "/config/route_ipv4", zebra_handler.clone());

    // NexusConfig init.
    let nexus_config = Arc::new(NexusConfig::new(nexus.clone(), mds));
    nexus_config.config_init();

    // NexusExec init.

    let uds_server = UdsServer::start(event_manager.clone(), nexus_config, &config_uds_path);
    nexus.set_config_server(uds_server);

    // Start nexus.
    match RouterNexus::start(nexus, event_manager) {
        Err(CoreError::SystemShutdown) => {
            info!("Nexus terminated")
        },
        _ => {
            error!("Nexus stopped unexpectedly")
        }
    }

    // Cleanup.
    if let Err(_) = fs::remove_file(config_uds_path) {

    }

    ()
}

