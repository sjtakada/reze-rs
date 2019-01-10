//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Simple Timer
//

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Instant;
use std::time::Duration;

use super::protocols::ProtocolType;
use super::master::ProtocolMaster;
use super::event;

// Timer entry
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Entry {
    pub protocol: ProtocolType,
    pub expiration: Instant,
    pub token: u32,
}

impl Ord for Entry {
    fn cmp(&self, other: &Entry) -> Ordering {
	other.expiration.cmp(&self.expiration)
    }
}

impl PartialOrd	for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
	Some(self.cmp(other))
    }
}

// Timer server
pub struct Server {
    heap: BinaryHeap<Entry>
}

impl Server {
    pub fn new() -> Server {
        Server { heap: BinaryHeap::new() }
    }

    pub fn register(&mut self, protocol: ProtocolType, d: Duration, token: u32) {
        let entry = Entry { protocol: protocol, expiration: Instant::now() + d, token: token };
        self.heap.push(entry);
    }

//    pub fn unregister(&self)

    pub fn pop_if_expired(&mut self) -> Option<Entry> {
        match self.heap.peek() {
            Some(entry) if entry.expiration < Instant::now() => {
                self.heap.pop()
            },
            _ => None,
        }
    }
}

// Timer client
pub struct Client {
    // Parent  
    master: RefCell<Arc<ProtocolMaster>>,

    // Token
    token: u32,

    // Token to EventHandler map
    timers: HashMap<u32, Arc<event::EventHandler + Send + Sync>>,
}

// Timer client implementation
impl Client {
    // Constructor
    pub fn new(master: Arc<ProtocolMaster>) -> Client {
        Client {
            master: RefCell::new(master),
            token: 0u32,
            timers: HashMap::new()
        }
    }

    pub fn register(&mut self, handler: Arc<event::EventHandler + Send + Sync>, _d: Duration) -> u32 {
        let token = self.token;
        self.timers.insert(token, handler);
        self.token += 1;

        token
    }

    pub fn unregister(&mut self, token: u32) -> Option<Arc<event::EventHandler + Send + Sync>> {
        self.timers.remove(&token)
    }

    // pub fn unregister()
}
