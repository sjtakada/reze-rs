//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//

use std::env;
use std::sync::Arc;

use log::info;
use simplelog::*;
use getopts::Options;

use routerd::core::event::*;
use routerd::core::nexus::*;
use routerd::core::uds_server::*;

const ROUTERD_VERSION: &str = "1.0";
const COPYRIGHT: &str = "Copyright (C) 2018,2019 Toshaki Takada";

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

