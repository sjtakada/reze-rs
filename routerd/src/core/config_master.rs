//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - config master.
//

use std::collections::HashMap;
use std::sync::Arc;

//use serde_json;

use super::config::Config;
use super::protocols::ProtocolType;

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

    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Master
    }

    fn register_child(&mut self, config: Arc<dyn Config + Sync + Send>) {
        self.map.insert(String::from(config.id()), config.clone());
    }

//    fn post(&self, path: &str, _params: Option<String>) -> Result<(), io::Error> {
//
//        Ok(())
//    }
}
