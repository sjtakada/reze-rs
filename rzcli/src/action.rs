//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Action.
//

use super::readline::*;

// Action trait.
pub trait CliAction {
    fn handle(&self) -> bool;
}

// Action mode.
pub struct CliActionMode {
    _name: String,
    _up: u64,
    _params: Vec<String>,
}

impl CliActionMode {
    pub fn new(value: &serde_json::Value) -> CliActionMode {
        let name = value["name"].as_str().unwrap_or("EXEX-MODE");
        let up = value["up"].as_u64().unwrap_or(0);
        let params = value["params"].as_object();

        CliActionMode {
            _name: String::from(name),
            _up: up,
            _params: Vec::new(),
        }
    }
}

impl CliAction for CliActionMode {
    fn handle(&self) -> bool {
        

        true
    }
}
