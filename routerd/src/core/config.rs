//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - Config.
//

use std::fmt;
use std::str::FromStr;
use std::rc::Rc;

use log::debug;
use regex::Regex;

//use super::protocols::ProtocolType;
use common::error::*;

#[derive(Copy, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
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
            "patch" => Ok(Method::Patch),
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
            Method::Patch => "PATCH",
        };

        write!(f, "{}", s)
    }
}

/// Config trait.
pub trait Config {
    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str;

    /// Register child Config trait object to this Config.
    fn register_child(&mut self, _child: Rc<dyn Config>) {
        debug!("This object does not have child Config object");
        ()
    }

    /// Lookup child Config with given path.
    fn lookup_child(&self, _path: &str) -> Option<Rc<dyn Config>> {
        debug!("Not implemented");
        None
    }

    /// Handle GET method.
    fn handle_get(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle POST method.
    fn handle_post(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle PUT method.
    fn handle_put(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle DELETE method.
    fn handle_delete(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Ok(())
    }

    /// Handle PATCH method.
    fn handle_patch(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Ok(())
    }
}

/// Utilities.
pub fn split_id_and_path(s: &str) -> Option<(String, Option<String>)> {
    let re = Regex::new(r"^/([^/]+)(.*)$").unwrap();
    match re.captures(s) {
        Some(caps) => {
            match caps.get(1) {
                Some(id) => {
                    let mut path: Option<String> = None;
                    if let Some(p) = caps.get(2) {
                        path = Some(p.as_str().to_string());
                    }

                    Some((id.as_str().to_string(), path))
                },
                None => None,
            }
        },
        None => None,
    }
}
