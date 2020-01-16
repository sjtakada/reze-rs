//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Simple Timer
//

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Instant;
use std::time::Duration;
use std::sync::Mutex;
use std::cmp::Ordering;

use common::event::*;

use super::master::ProtocolMaster;

/// TimerHandler trait
pub trait TimerHandler: EventHandler {

    /// Get expiration time.
    fn expiration(&self) -> Instant;

    /// Set expiration time.
    fn set_expiration(&mut self, d: Duration) -> ();
}

impl Ord for dyn TimerHandler {
    fn cmp(&self, other: &Self) -> Ordering {
	other.expiration().cmp(&self.expiration())
    }
}

impl PartialOrd for dyn TimerHandler {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
	Some(self.cmp(other))
    }
}

impl Eq for dyn TimerHandler {
}

impl PartialEq for dyn TimerHandler {
    fn eq(&self, other: &Self) -> bool {
        other.expiration() == self.expiration()
    }
}

/// Timer server
pub struct Server {

    /// Ordering handler by expiration time.
    heap: RefCell<BinaryHeap<Rc<dyn TimerHandler>>>
}

impl Server {
    pub fn new() -> Server {
        Server {
            heap: RefCell::new(BinaryHeap::new())
        }
    }

    pub fn register(&self, d: Duration, mut handler: Rc<dyn TimerHandler>) {
        Rc::get_mut(&mut handler).unwrap().set_expiration(d);
        self.heap.borrow_mut().push(handler);
    }

    pub fn pop_if_expired(&mut self) -> Option<Rc<dyn TimerHandler>> {
        match self.heap.borrow_mut().peek() {
            Some(handler) if handler.expiration() < Instant::now() => {
                self.heap.borrow_mut().pop()
            },
            _ => None,
        }
    }

    pub fn run(&mut self) {
        while let Some(handler) = self.pop_if_expired() {
            let _ = handler.handle(EventType::TimerEvent);
        }
    }
}

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
