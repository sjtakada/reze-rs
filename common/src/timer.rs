//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Simple Timer
// - TimerServer  .. High level timer management
// - TimerHandler .. EventHandler kind to handle client job.
//

use std::collections::BinaryHeap;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use std::time::Duration;
use std::cmp::Ordering;

use super::event::*;


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

/// Timer server.
pub struct TimerServer {

    /// Ordering handler by expiration time.
    heap: RefCell<BinaryHeap<Rc<dyn TimerHandler>>>
}

impl TimerServer {
    pub fn new() -> TimerServer {
        TimerServer {
            heap: RefCell::new(BinaryHeap::new())
        }
    }

    pub fn register(&self, d: Duration, mut handler: Rc<dyn TimerHandler>) {
        Rc::get_mut(&mut handler).unwrap().set_expiration(d);
        self.heap.borrow_mut().push(handler);
    }

    pub fn pop_if_expired(&mut self) -> Option<Rc<dyn TimerHandler>> {
        if match self.heap.borrow_mut().peek() {
            Some(handler) if handler.expiration() < Instant::now() => true,
            _ => false,
        } {
            self.heap.borrow_mut().pop()
        } else {
            None
        }
    }

    pub fn run(&mut self) {
        while let Some(handler) = self.pop_if_expired() {
            let _ = handler.handle(EventType::TimerEvent);
        }
    }
}

