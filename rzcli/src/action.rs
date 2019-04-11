//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Action.
//

use super::cli::Cli;
//use super::readline::*;
use super::error::CliError;

// Action trait.
pub trait CliAction {
    fn handle(&self, cli: &Cli) -> Result<(), CliError>;
}

// Action mode.
pub struct CliActionMode {
    name: String,
    _up: u64,
    _params: Vec<String>,
}

impl CliActionMode {
    pub fn new(value: &serde_json::Value) -> CliActionMode {
        let name = value["name"].as_str().unwrap_or("EXEX-MODE");
        let up = value["up"].as_u64().unwrap_or(0);
        let params = value["params"].as_object();

        CliActionMode {
            name: String::from(name),
            _up: up,
            _params: Vec::new(),
        }
    }
}

impl CliAction for CliActionMode {
    fn handle(&self, cli: &Cli) -> Result<(), CliError> {
        cli.set_mode(&self.name)?;

        Ok(())
    }
}
