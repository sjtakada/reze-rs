//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - config master.
//

use std::rc::Rc;
use std::collections::HashMap;

use log::error;

use super::error::*;
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

    pub fn lookup(&self, path: &str) -> Option<&ConfigOrProtocol> {
        if let Some((id, _path)) = split_id_and_path(path) {
            self.map.get(&id)
        } else {
            None
        }
    }

    pub fn apply(&self, method: Method, path: &str, body: Option<Box<String>>) -> Result<(), CoreError> {
        if let Some((id, path)) = split_id_and_path(path) {
            match path {
                Some(path) => {
                    if let Some(config_or_protocol) = self.map.get(&id) {
                        match config_or_protocol {
                            ConfigOrProtocol::Local(config) => {
                                match method {
                                    Method::Get => config.get(&path, body),
                                    Method::Post => config.post(&path, body),
                                    Method::Put => config.put(&path, body),
                                    Method::Delete => config.delete(&path, body),
                                    Method::Patch => config.patch(&path, body),
                                }
                            },
                            _ => {
                                // some error
                                error!("No local config to apply");
                                Err(CoreError::ConfigNotFound("No local config".to_string()))
                            }
                        }
                    } else {
                        error!("No matched path to apply {}", path);
                        Err(CoreError::ConfigNotFound("No match path".to_string()))
                    }
                },
                None => {
                    error!("Insufficient config path");
                    Err(CoreError::ConfigNotFound("Insufficient config path".to_string()))
                }
            }
        } else {
            error!("Invalid path");
            Err(CoreError::ConfigNotFound("Invalid path".to_string()))
        }
    }

    pub fn register_protocol(&mut self, path: &str, protocol_type: ProtocolType) {
        self.map.insert(path.to_string(), ConfigOrProtocol::Proto(protocol_type));
    }

    pub fn register_config(&mut self, path: &str, config: Rc<dyn Config>) {
        self.map.insert(path.to_string(), ConfigOrProtocol::Local(config));
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
