//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Simple Timer
//

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::sync::Arc;
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
    pub token: i32,
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

    pub fn register(&mut self, protocol: ProtocolType, d: Duration, token: i32) {
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
//    master: ProtocolMaster,

    token: u32,

    timers: HashMap<u32, Arc<event::EventHandler + Send + Sync>>,
}

impl Client {
    pub fn new() -> Client {
        Client {
//            master: master,
            token: 0u32,
            timers: HashMap::new()
        }
    }

    pub fn register(&mut self, handler: Arc<event::EventHandler + Send + Sync>, d: Duration) {
        self.timers.insert(self.token, handler);
        self.token += 1;
    }

    // pub fn unregister()
}
