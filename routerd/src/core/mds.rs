//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - Manageement Data Store.
//

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use log::debug;
use regex::Regex;

use common::error::*;

/// Management Data Store node.
///  Store leaf node and handler.
pub struct MdsNode {

    /// Map for children nodes.
    children: RefCell<HashMap<String, Rc<MdsNode>>>,

    /// Mds Handler.
    handler: RefCell<Option<Rc<dyn MdsHandler>>>,
}

/// MdsNode implementation.
impl MdsNode {

    /// Constructor.
    pub fn new() -> MdsNode {
        MdsNode {
            children: RefCell::new(HashMap::new()),
            handler: RefCell::new(None),
        }
    }

    /// Set handler to this node.
    pub fn set_handler(&self, handler: Rc<dyn MdsHandler>) {
        self.handler.borrow_mut().replace(handler);
    }

    /// Return direct child node with given name.
    pub fn lookup_child(&self, name: &str) -> Option<Rc<MdsNode>> {
        match self.children.borrow().get(name) {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }

    /// Register a single node as a child.
    fn register(&self, name: &str, node: Rc<MdsNode>) -> Rc<MdsNode> {
        self.children.borrow_mut().insert(name.to_string(), node.clone());
        node
    }

    /// Register handler.
    /// Create as many intermediate nodes if needed.
    pub fn register_handler(mut curr: Rc<MdsNode>, path: &str, handler: Rc<dyn MdsHandler>) {
        let path = path.trim_matches('/');
        let v: Vec<&str> = path.split('/').collect();

        for p in v {
            curr = match curr.lookup_child(p) {
                Some(child) => child,
                None => curr.register(p, Rc::new(MdsNode::new())),
            };
        }

        curr.set_handler(handler);
    }

    /// Lookup handler.
    pub fn lookup_handler(mut curr: Rc<MdsNode>, path: &str) -> Option<Rc<dyn MdsHandler>> {
        let path = path.trim_matches('/');
        let v: Vec<&str> = path.split('/').collect();

        for p in v {
            curr = match curr.lookup_child(p) {
                Some(child) => child,
                None => return None,
            };
        }

        match *curr.handler.borrow_mut() {
            Some(ref mut handler) => Some(handler.clone()),
            None => None,
        }
    }
}

/// Management Data Store trait.
///  Store config or device state in hierarchy.
///  Dispatch REST style request to handler.
pub trait MdsHandler {

    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str {
        "placeholder"
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
    let re = Regex::new(r"^/*([^/]+)(.*)$").unwrap();
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


///
/// Unit tests.
///
#[cfg(test)]
mod tests {
    use super::*;

    pub struct Handler {
    }

    impl MdsHandler for Handler {
    }

    #[test]
    pub fn test_mds_node() {
        let handler = Rc::new(Handler {});
        let root = Rc::new(MdsNode::new());

        MdsNode::register_handler(root.clone(), "/show/ip/route", handler.clone());
        MdsNode::register_handler(root.clone(), "/show/ip/route/summary", handler.clone());
        MdsNode::register_handler(root.clone(), "/show/ipv6/route", handler.clone());

        match MdsNode::lookup_handler(root.clone(), "/show/ip/route/") {
            Some(_) => {},
            None => assert!(false),
        }

        match MdsNode::lookup_handler(root.clone(), "/show/ip/route") {
            Some(_) => {},
            None => assert!(false),
        }

        match MdsNode::lookup_handler(root.clone(), "show/ip/route") {
            Some(_) => {},
            None => assert!(false),
        }

        match MdsNode::lookup_handler(root.clone(), "show/ip/rout") {
            Some(_) => assert!(false),
            None => {}
        }

        match MdsNode::lookup_handler(root.clone(), "/show/ip/route/summary") {
            Some(_) => {}
            None => assert!(false),
        }

        match MdsNode::lookup_handler(root.clone(), "/show/ipv6/route/summary") {
            Some(_) => assert!(false),
            None => {}
        }
    }
}
