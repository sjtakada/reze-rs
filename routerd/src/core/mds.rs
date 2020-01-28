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
use common::method::Method;

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

    /// Return true if the node has child.
    pub fn has_child(&self) -> bool {
        self.children.borrow().len() > 0
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

            if !curr.has_child() {
                break;
            }
        }

        match *curr.handler.borrow_mut() {
            Some(ref mut handler) => Some(handler.clone()),
            None => None,
        }
    }

    /// Handle request.
    pub fn handle(curr: Rc<MdsNode>, id: u32, method: Method, path: &str, body: Option<Box<String>>) -> Result<(), CoreError> {
        if let Some(handler) = MdsNode::lookup_handler(curr, path) {
            if handler.is_generic() {
                if let Err(err) = handler.handle_generic(id, method, path, body) {
                    Err(err)
                } else {
                    Ok(())
                }
            } else {
                match method {
                    Method::Get => handler.handle_get(&path, body),
                    Method::Post => handler.handle_post(&path, body),
                    Method::Put => handler.handle_put(&path, body),
                    Method::Delete => handler.handle_delete(&path, body),
                    Method::Patch => handler.handle_patch(&path, body),
                }
            }
        } else {
            Err(CoreError::MdsNoHandler)
        }
    }
}

/// Management Data Store trait.
///  Store config or device state in hierarchy.
///  Dispatch REST style request to handler.
pub trait MdsHandler {

    /// Return handle_generic implmented.
    fn is_generic(&self) -> bool {
        false
    }

    /// Handle method generic.
    fn handle_generic(&self, _id: u32, _method: Method, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        Err(CoreError::NotImplemented)
    }

    /// Handle GET method.
    fn handle_get(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Err(CoreError::NotImplemented)
    }

    /// Handle POST method.
    fn handle_post(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Err(CoreError::NotImplemented)
    }

    /// Handle PUT method.
    fn handle_put(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Err(CoreError::NotImplemented)
    }

    /// Handle DELETE method.
    fn handle_delete(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Err(CoreError::NotImplemented)
    }

    /// Handle PATCH method.
    fn handle_patch(&self, _path: &str, _params: Option<Box<String>>) -> Result<(), CoreError> {
        debug!("Method not implemented");
        Err(CoreError::NotImplemented)
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
            Some(_) => {}
            None => assert!(false),
        }
    }
}
