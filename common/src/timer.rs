//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Simple Timer
// - TimerServer  .. High level timer management
// - TimerHandler .. EventHandler kind to handle client job.
//

use std::collections::BinaryHeap;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Instant;
use std::time::Duration;
use std::cmp::Ordering;

use log::error;

use super::event::*;

/// TimerHandler trait.
pub trait TimerHandler: EventHandler
where Self: Send,
      Self: Sync
{
    /// Get expiration time.
    fn expiration(&self) -> Instant;

    /// Set expiration time.
    fn set_expiration(&self, d: Duration) -> ();
}

/// Ord implementation for TimerHandler.
impl Ord for dyn TimerHandler {
    fn cmp(&self, other: &Self) -> Ordering {
	other.expiration().cmp(&self.expiration())
    }
}

/// PartialOrd implementation for TimerHandler.
impl PartialOrd for dyn TimerHandler {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
	Some(self.cmp(other))
    }
}

/// Eq implementation for TimerHandler.
impl Eq for dyn TimerHandler {
}

/// PartialEq implementation for TimerHandler.
impl PartialEq for dyn TimerHandler {
    fn eq(&self, other: &Self) -> bool {
        other.expiration() == self.expiration()
    }
}

/// Timer server.
pub struct TimerServer {

    /// Ordering handler by expiration time.
    heap: RefCell<BinaryHeap<Arc<dyn TimerHandler>>>
}

/// TimerServer implementation.
impl TimerServer {

    /// Constructor.
    pub fn new() -> TimerServer {
        TimerServer {
            heap: RefCell::new(BinaryHeap::new())
        }
    }

    /// Register timer handler.
    pub fn register(&self, d: Duration, handler: Arc<dyn TimerHandler>) {
        handler.set_expiration(d);
        self.heap.borrow_mut().push(handler);
    }

    /// Pop a timer handler if it is expired.
    fn pop_if_expired(&self) -> Option<Arc<dyn TimerHandler>> {
        if match self.heap.borrow_mut().peek() {
            Some(handler) if handler.expiration() < Instant::now() => true,
            _ => false,
        } {
            self.heap.borrow_mut().pop()
        } else {
            None
        }
    }

    /// Run all expired event handler.
    pub fn run(&self) -> Option<Arc<dyn TimerHandler>> {
        self.pop_if_expired()
    }
}

