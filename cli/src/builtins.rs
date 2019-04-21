//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Builtin functions.
//

use super::cli::Cli;
use super::error::CliError;

pub fn help(cli: &Cli, _params: &Vec<String>) -> Result<(), CliError> {
    println!(r#"Help may be requested at any point in a command by entering
a question mark '?'.  If nothing matches, the help list will
be empty and you must backup until entering a '?' shows the
available options.
Two styles of help are provided:
1. Full help is available when you are reaady to enter a
   command argument (e.g. 'show ?') and describes each possible
   argument.
2. Partial help is provided when an abbreviated argument is entered
   and you want to know what arguments match the input
   (e.g. 'show pr?'.)
"#);
    Ok(())
}

pub fn exit(cli: &Cli, _params: &Vec<String>) -> Result<(), CliError> {
    cli.set_mode_up()?;

    Ok(())
}

pub fn enable(cli: &Cli, _params: &Vec<String>) -> Result<(), CliError> {
    cli.set_privilege(15);
    cli.set_prompt();

    Ok(())
}

pub fn disable(cli: &Cli, _params: &Vec<String>) -> Result<(), CliError> {
    cli.set_privilege(1);
    cli.set_prompt();

    Ok(())
}

pub fn show_privilege(cli: &Cli, _params: &Vec<String>) -> Result<(), CliError> {
    println!("Current privilege level is {}", cli.privilege());
    Ok(())
}
