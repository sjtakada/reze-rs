//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Action.
//

use std::collections::HashMap;
use regex::Regex;

use serde_json;

use super::cli::Cli;
use super::error::CliError;
use super::node::Value;

// Action trait.
pub trait CliAction {
    fn handle(&self, cli: &Cli, params: &HashMap<String, Value>) -> Result<(), CliError>;
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
    fn handle(&self, cli: &Cli, _params: &HashMap<String, Value>) -> Result<(), CliError> {
        cli.set_mode(&self.name)?;

        Ok(())
    }
}

pub enum CliViewTemplate {
    Internal,
    External,
}

// Http action.
pub struct CliActionRemote {
    method: String,
    target: String,
    path: String,
    params: String,
    view: Option<(CliViewTemplate, String)>,
}

impl CliActionRemote {
    pub fn new(value: &serde_json::Value) -> CliActionRemote {
        let method = value["method"].as_str().unwrap_or("GET");
        let target = value["target"].as_str().unwrap_or("config");
        let path = value["path"].as_str().unwrap_or("");
        let params = value["params"].to_string();
        let view = if value["view"].is_object() {
            let v = value["view"].as_object().unwrap();

            if v["template"].is_string() && v["source"].is_string() {
                let source = v["source"].as_str().unwrap();
                match v["template"].as_str().unwrap() {
                    "internal" => Some((CliViewTemplate::Internal, source.to_string())),
                    "external" => Some((CliViewTemplate::External, source.to_string())),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        CliActionRemote {
            method: String::from(method),
            target: String::from(target),
            path: String::from(path),
            params: params,
            view: view,
        }
    }
}

impl CliAction for CliActionRemote {
    fn handle(&self, cli: &Cli, params: &HashMap<String, Value>) -> Result<(), CliError> {

        let remote_client = match cli.remote_client(&self.target) {
            Some(remote_client) => remote_client,
            None => return Err(CliError::ActionError(format!("No remote defined for {:?}", self.target)))
        };
        let remote_prefix = remote_client.prefix();

        // Replace path with params.
        let path = self.path.split('/').map(|p| {
            if &p[0..1] == ":" {
                match params.get(&p[1..]) {
                    Some(v) => format!("{}", v),
                    None => "".to_string(),
                }
            } else {
                p.to_string()
            }
        }).collect::<Vec<String>>().join("/");

        let path = format!("{}/{}", remote_prefix, path);

        // TODO: Maybe we could just check first letter of keyword instead of using Regex..
        let re = Regex::new(r"^:[A-Z]").unwrap();

        let mut body = self.params.clone();
        for (k, v) in params {
            let key = format!(":{}", k);

            if re.is_match(&key) {
                body = body.replace(&key, &v.to_string());
            }
        }

        // build json body.
        let request = format!("{} {}\n\n{}", self.method, path, body);

        // If only debug.
        if cli.is_debug() {
            println!("% Request to {}", self.target);
            println!("{}", request);
        }

        cli.remote_send(&self.target, &request);

        let resp = cli.remote_recv(&self.target);
        if cli.is_debug() {
            println!("% Response");
            println!("{:?}", resp);
        }

        if let Some((template, name)) = &self.view {
            if let Some(json_str) = resp {

                match serde_json::from_str(&json_str) {
                    Ok(value) => {
                        match template {
                            CliViewTemplate::Internal => {
                                cli.view().call(&name, &value);
                            },
                            CliViewTemplate::External => {
                            },
                        }
                    },
                    Err(err) => {
                        println!("Unable to parse response from server {:?} {:?}", err, json_str);
                    },
                }
            }
        }

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
    fn handle(&self, cli: &Cli, _params: &HashMap<String, Value>) -> Result<(), CliError> {
        cli.call_builtin(&self.func, &self.params)
    }
}

