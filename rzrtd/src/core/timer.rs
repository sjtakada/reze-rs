//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Simple Timer
//

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;
use std::time::Duration;

use super::protocols::ProtocolType;

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

