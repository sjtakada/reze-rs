//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Main
//

use std::env;
use std::path::PathBuf;

use getopts::Options;

use common::consts::COPYRIGHT;

use cli::utils::*;
use cli::master::CliMaster;
use cli::config::Config;


const CLI_VERSION: &str = "1.0";

/// Show help.
fn print_help(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

/// Show version.
fn print_version(program: &str) {
    println!("{} version {}", program, CLI_VERSION);
    println!("{}", COPYRIGHT);
    println!("");
}

/// Main.
fn main() {
    // Command line arguments.
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("d", "debug", "Runs in debug mode");
    opts.optflag("c", "config", "Meta config file for CLI");
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

    let config_file = match matches.opt_str("c") {
        Some(config_file) => config_file.to_string(),
        None => "reze_cli_config.json".to_string(),
    };

    // Read and parse config fiile.
    let path = PathBuf::from(config_file);
    let config = match json_read(&path) {
        Some(json) => Config::from_json(&json),
        None => Config::new(),
    };

    match CliMaster::start(config) {
        Ok(_) => {},
        Err(err) => panic!("CLI Init error: {}", err),
    };
}
