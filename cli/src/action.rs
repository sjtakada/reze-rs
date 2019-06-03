//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Action.
//

use super::cli::Cli;
use super::error::CliError;

// Action trait.
pub trait CliAction {
    fn handle(&self, cli: &Cli) -> Result<(), CliError>;
}

// Mode action.
pub struct CliActionMode {
    name: String,
    _up: u64,
    _params: Vec<String>,
}

impl CliActionMode {
    pub fn new(value: &serde_json::Value) -> CliActionMode {
        let name = value["name"].as_str().unwrap_or("EXEX-MODE");
        let up = value["up"].as_u64().unwrap_or(0);
        let _params = value["params"].as_object();

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

// Built-in action.
pub struct CliActionBuiltin {
    func: String,
    params: Vec<String>,
}

impl CliActionBuiltin {
    pub fn new(value: &serde_json::Value) -> CliActionBuiltin {
        let func = value["func"].as_str().unwrap();

        CliActionBuiltin {
            func: String::from(func),
            params: Vec::new(),
        }
    }
}

impl CliAction for CliActionBuiltin {
    fn handle(&self, cli: &Cli) -> Result<(), CliError> {
        cli.call_builtin(&self.func, &self.params)
    }
}

