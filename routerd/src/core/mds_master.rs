//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - Management Data Store Master.
//

use std::rc::Rc;
use std::collections::HashMap;

use log::error;

use common::error::*;
use common::method::Method;

use super::mds::*;
use super::protocols::ProtocolType;

/// Config or Protocol
pub enum ConfigOrProtocol {
    Local(Rc<dyn MdsHandler>),
    Proto(ProtocolType),
}

/// Config master
pub struct MdsMaster {

    /// Top level config storage.
    map: HashMap<String, ConfigOrProtocol>,
}

/// MdsMaster implementation.
impl MdsMaster {

    /// Constructor.
    pub fn new() -> MdsMaster {
        MdsMaster {
            map: HashMap::new(),
        }
    }

    /// Lookup config with path.
    pub fn lookup(&self, path: &str) -> Option<&ConfigOrProtocol> {
        if let Some((id, _path)) = split_id_and_path(path) {
            self.map.get(&id)
        } else {
            None
        }
    }

    /// Deliver config to given path.
    pub fn apply(&self, method: Method, path: &str, body: Option<Box<String>>) -> Result<(), CoreError> {
        if let Some((id, path)) = split_id_and_path(path) {
            match path {
                Some(path) => {
                    if let Some(config_or_protocol) = self.map.get(&id) {
                        match config_or_protocol {
                            ConfigOrProtocol::Local(config) => {
                                match method {
                                    Method::Get => config.handle_get(&path, body),
                                    Method::Post => config.handle_post(&path, body),
                                    Method::Put => config.handle_put(&path, body),
                                    Method::Delete => config.handle_delete(&path, body),
                                    Method::Patch => config.handle_patch(&path, body),
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

    pub fn register_config(&mut self, path: &str, config: Rc<dyn MdsHandler>) {
        self.map.insert(path.to_string(), ConfigOrProtocol::Local(config));
    }
}

/*
impl MdsHandler for MdsMaster {
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
