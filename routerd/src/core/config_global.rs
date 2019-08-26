//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - global config.
//

//use std::io;
//use std::net::{Ipv4Addr, Ipv6Addr};
use std::collections::HashMap;
//use std::rc::Rc;
use std::sync::Arc;

//use serde_json;
//use log::debug;
//use jsonpath;

//use rtable::prefix::*;
use super::config::Config;

/// Global config.
pub struct ConfigGlobal {
    /// Top level config storage.
    map: HashMap<String, Arc<Config + Send + Sync>>,
}

impl ConfigGlobal {
    pub fn new() -> ConfigGlobal {
        ConfigGlobal {
            map: HashMap::new(),
        }
    }
}

impl Config for ConfigGlobal {
    fn id(&self) -> &str {
        "config"
    }

    fn register_child(&mut self, config: Arc<Config + Sync + Send>) {
        self.map.insert(String::from(config.id()), config.clone());
    }
}
