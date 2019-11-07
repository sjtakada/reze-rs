//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - config master.
//

use std::collections::HashMap;
use std::rc::Rc;

use super::config::*;
use super::protocols::ProtocolType;

/// Config or Protocol
pub enum ConfigOrProtocol {
    Local(Rc<dyn Config>),
    Proto(ProtocolType),
}

/// Global config.
pub struct ConfigMaster {
    /// Top level config storage.
    map: HashMap<String, ConfigOrProtocol>,
}

impl ConfigMaster {
    pub fn new() -> ConfigMaster {
        ConfigMaster {
            map: HashMap::new(),
        }
    }

    pub fn lookup_child(&self, path: &str) -> Option<&ConfigOrProtocol> {
        if let Some((id, path)) = split_id_and_path(path) {
            self.map.get(&id)
        } else {
            None
        }
    }
}

/*
impl Config for ConfigMaster {
    fn id(&self) -> &str {
        "config"
    }

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Master
    }

    fn register_child(&mut self, config: Rc<dyn Config>) {
        self.map.insert(String::from(config.id()), config.clone());
    }

    fn lookup_child(&self, path: &str) -> Option<Rc<dyn Config>> {
        if let Some((id, path)) = split_id_and_path(path) {
            match self.map.get(&id) {
                Some(config) => Some(config.clone()),
                None => None,
            }
        } else {
            None
        }
    }
}
*/
