//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Simple Timer
//

use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Duration;
use std::sync::Mutex;

use common::event::*;

use super::master::ProtocolMaster;

/// Timer client
pub struct Client {

    /// Parent  
    _master: RefCell<Arc<ProtocolMaster>>,

    /// Token
    token: u32,

    /// Token to EventHandler map
    timers: Mutex<HashMap<u32, Arc<dyn EventHandler + Send + Sync>>>,
}

/// Timer client implementation
impl Client {

    /// Constructor
    pub fn new(master: Arc<ProtocolMaster>) -> Client {
        Client {
            _master: RefCell::new(master),
            token: 0u32,
            timers: Mutex::new(HashMap::new())
        }
    }

    pub fn register(&mut self, handler: Arc<dyn EventHandler + Send + Sync>, _d: Duration) -> u32 {
        let token = self.token;
        let mut timers = self.timers.lock().unwrap();
        timers.insert(token, handler);
        self.token += 1;

        token
    }

    pub fn unregister(&mut self, token: u32) -> Option<Arc<dyn EventHandler + Send + Sync>> {
        let mut timers = self.timers.lock().unwrap();
        timers.remove(&token)
    }
}
