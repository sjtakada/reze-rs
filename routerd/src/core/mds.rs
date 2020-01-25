//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - Manageement Data Store.
//

use std::rc::Rc;

use log::debug;
use regex::Regex;

use common::error::*;

/// Management Data Store trait.
///  Store config or device state in hierarchy.
///  Dispatch REST like request to certain data objects.
pub trait MdsHandler {

    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str;

    /// Register child MdsHandler trait object to this Config.
    fn register_child(&mut self, _child: Rc<dyn MdsHandler>) {
        debug!("This object does not have child MdsHandler object");
        ()
    }

    /// Lookup child MdsHandler with given path.
    fn lookup_child(&self, _path: &str) -> Option<Rc<dyn MdsHandler>> {
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
