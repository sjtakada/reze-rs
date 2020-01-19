//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Simple Timer
// - TimerServer  .. High level timer management
// - TimerHandler .. EventHandler kind to handle client job.
//

use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::Instant;
use std::time::Duration;
use std::cmp::Ordering;

use super::event::*;


/// TimerHandler trait.
pub trait TimerHandler: EventHandler
where Self: Send,
      Self: Sync
{
    /// Get expiration time.
    fn expiration(&self) -> Instant;

    /// Set expiration time.
    fn set_expiration(&mut self, d: Duration) -> ();
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
    heap: BinaryHeap<Arc<dyn TimerHandler>>
}

/// TimerServer implementation.
impl TimerServer {

    /// Constructor.
    pub fn new() -> TimerServer {
        TimerServer {
            heap: BinaryHeap::new()
        }
    }

    /// Register timer handler.
    pub fn register(&mut self, d: Duration, mut handler: Arc<dyn TimerHandler>) {
        Arc::get_mut(&mut handler).unwrap().set_expiration(d);
        self.heap.push(handler);
    }

    /// Pop a timer handler if it is expired.
    fn pop_if_expired(&mut self) -> Option<Arc<dyn TimerHandler>> {
        if match self.heap.peek() {
            Some(handler) if handler.expiration() < Instant::now() => true,
            _ => false,
        } {
            self.heap.pop()
        } else {
            None
        }
    }

    /// Run all expired event handler.
    pub fn run(&mut self) {
        while let Some(handler) = self.pop_if_expired() {
            let _ = handler.handle(EventType::TimerEvent);
        }
    }
}

