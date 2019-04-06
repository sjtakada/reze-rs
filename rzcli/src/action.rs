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
    name: String,
    up: u64,
    params: Vec<String>,
}

impl CliActionMode {
    pub fn new(value: &serde_json::Value) -> CliActionMode {
        let name = value["name"].as_str().unwrap_or("EXEX-MODE");
        let up = value["up"].as_u64().unwrap_or(0);
        let params = value["params"].as_object();

        CliActionMode {
            name: String::from(name),
            up: up,
            params: Vec::new(),
        }
    }
}

impl CliAction for CliActionMode {
    fn handle(&self) -> bool {
        

        true
    }
}
