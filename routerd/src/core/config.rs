//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core - config.
//

use std::io;
use std::fmt;
use std::str::FromStr;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use serde_json;
use log::debug;

use rtable::prefix::*;

use super::protocols::ProtocolType;
use super::error::*;

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Update,
}

impl FromStr for Method {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let method = s.to_lowercase();

        match method.as_ref() {
            "get" => Ok(Method::Get),
            "post" => Ok(Method::Post),
            "put" => Ok(Method::Put),
            "delete" => Ok(Method::Delete),
            "update" => Ok(Method::Update),
            _ => Err(CoreError::ParseMethod),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Update => "UPDATE",
        };

        write!(f, "{}", s)
    }
}

/*
pub enum Key {
    Singular,
    Str(String),
    Num(u32),
    Address4(Ipv4Addr),
    Address6(Ipv6Addr),
    Prefix4(Prefix<Ipv4Addr>),
    Prefix6(Prefix<Ipv6Addr>),
}
*/

/// Config trait.
pub trait Config {
    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str;

    /// Return protocol type, owner of the config.
    fn protocol_type(&self) -> ProtocolType;

    /// Register child Config trait object to this Config.
    fn register_child(&mut self, _child: Arc<dyn Config + Send + Sync>) {
        debug!("This object does not have child Config object");
        ()
    }

    /// Handle GET method.
    fn get(&self, _path: &str, _params: Option<String>) -> Result<(), io::Error> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle POST method.
    fn post(&self, _path: &str, _params: Option<String>) -> Result<(), io::Error> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle PUT method.
    fn put(&self, _path: &str, _params: Option<String>) -> Result<(), io::Error> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle DELETE method.
    fn delete(&self, _path: &str, _params: Option<String>) -> Result<(), io::Error> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle UPDATE method.
    fn update(&self, _path: &str, _params: Option<String>) -> Result<(), io::Error> {
        debug!("Method not implemented");
        Ok(())
    }
}
