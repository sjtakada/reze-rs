//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Main
//

use std::env;
use getopts::Options;

use rzcli::cli::Cli;


const CLI_VERSION: &str = "1.0";
const COPYRIGHT: &str = "Copyright (C) 2018,2019 Toshiaki Takada";

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
    opts.optopt("j", "json", "Set CLI JSON def directory", "DIR");
    opts.optflag("d", "debug", "Runs in debug mode");
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

    let cli = Cli::new();
    // TBD: set configuration
    match cli.init() {
        Ok(_) => {
        }
        Err(_err) => {
        }
    }

    cli.run();
}
