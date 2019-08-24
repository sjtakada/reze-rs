//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - config.
//

use std::io;
use serde_json;

/// Config trait.
pub trait Config {
    /// Handle GET method.
    fn get(&self, _path: &str, _params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        Ok(())
    }

    /// Handle POST method.
    fn post(&self, _path: &str, _params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        Ok(())
    }

    /// Handle PUT method.
    fn put(&self, _path: &str, _params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        Ok(())
    }

    /// Handle DELETE method.
    fn delete(&self, _path: &str, _params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        Ok(())
    }

    /// Handle UPDATE method.
    fn update(&self, _path: &str, _params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        Ok(())
    }
}
