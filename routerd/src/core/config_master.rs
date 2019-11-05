//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - config master.
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
pub struct ConfigMaster {
    /// Top level config storage.
    map: HashMap<String, Arc<dyn Config + Send + Sync>>,
}

impl ConfigMaster {
    pub fn new() -> ConfigMaster {
        ConfigMaster {
            map: HashMap::new(),
        }
    }
}

impl Config for ConfigMaster {
    fn id(&self) -> &str {
        "config"
    }

    fn register_child(&mut self, config: Arc<dyn Config + Sync + Send>) {
        self.map.insert(String::from(config.id()), config.clone());
    }
}