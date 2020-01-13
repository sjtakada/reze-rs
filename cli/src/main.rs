//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Main
//

use std::env;
use getopts::Options;

use cli::cli::Cli;
//use cli::error::CliError;

use cli::config::Config;

const CLI_VERSION: &str = "1.0";
const COPYRIGHT: &str = "Copyright (C) 2018-2020 Toshiaki Takada";

// Help.
fn print_help(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

// Version.
fn print_version(program: &str) {
    println!("{} version {}", program, CLI_VERSION);
    println!("{}", COPYRIGHT);
    println!("");
}

//
fn main() {
    // Command line arguments.
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("d", "debug", "Runs in debug mode");
    opts.optopt("j", "json", "Set CLI JSON def directory", "DIR");
    opts.optopt("s", "server", "Set API server IP address", "SERVER-IP");
    opts.optopt("p", "prefix", "Set API path prefix", "API-PREFIX");
    opts.optopt("u", "user", "Set username and password to authenticate server", "USERNAME:PASSWORD");
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

    let mut config = Config::new();
        
    if let Some(json) = matches.opt_str("j") {
        config.set_json(&json);
    }

    if let Some(server) = matches.opt_str("s") {
        config.set_server_ip(&server);
    }

    if let Some(prefix) = matches.opt_str("p") {
        config.set_api_prefix(&prefix);
    }

    if let Some(user) = matches.opt_str("u") {
        config.set_user_pass(&user);
    }

    if matches.opt_present("d") {
        config.set_debug(true);
    }

    let mut cli = Cli::new();
    match cli.init(config) {
        Ok(_) => {},
        Err(err) => panic!("CLI Init error: {}", err),
    };
}
