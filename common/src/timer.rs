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
use std::sync::Mutex;
use std::time::Instant;
use std::time::Duration;
use std::cmp::Ordering;
use std::future::Future;
use std::task::Context;
use std::task::Poll;
use std::pin::Pin;

use futures::future::BoxFuture;
use futures::future::FutureExt;
use futures::task::ArcWake;
use futures::task::waker_ref;

//use log::error;

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

/* XXXX */

/// Task, wrapping a Future.
pub struct TimerTask {

    /// Wrapped future.
    pub future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Expiration time.
    fire_at: Instant,

    /// Canceled flag.
    canceled: bool,
}

impl Ord for TimerTask {
    fn cmp(&self, other: &Self) -> Ordering {
        other.fire_at.cmp(&self.fire_at)
    }
}

impl PartialOrd for TimerTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TimerTask {
}

impl PartialEq for TimerTask {
    fn eq(&self, other: &Self) -> bool {
        other.fire_at == self.fire_at
    }
}


impl ArcWake for TimerTask {
    fn wake_by_ref(_arc_self: &Arc<Self>) {

    }
}

/// Timer Event Manager with Future.
pub struct TimerEventManager {

    /// Ordering TimerTask by expiration time.
    heap: BinaryHeap<Arc<TimerTask>>
}

impl TimerEventManager {

    /// Constructor.
    pub fn new() -> TimerEventManager {
        TimerEventManager {
            heap: BinaryHeap::new(),
        }
    }

    /// Register Timer.
    pub fn register_timer(&mut self, duration: Duration, 
                          f: Box<dyn Fn() + 'static + Send>) -> Arc<TimerTask>
    {
        let task = self.get_timer_task(duration, async move {
            TimerFuture::new(duration).await;
            f();
        });
        self.heap.push(task.clone());

        task
    }

    /// Get Timer task.
    fn get_timer_task(&self, duration: Duration,
                      future: impl Future<Output = ()> + 'static + Send) -> Arc<TimerTask>
    {
        let future = future.boxed();
        Arc::new(TimerTask {
            future: Mutex::new(Some(future)),
            fire_at: Instant::now() + duration,
            canceled: false,
        })
    }

    fn peek(&mut self) -> Option<Arc<TimerTask>> {
        match self.heap.peek() {
            Some(task) => Some(task.clone()),
            None => None
        }
    }

    pub fn run_task(task: Arc<TimerTask>) {

    }

    pub fn _run(&mut self) {
        while let Some(task) = self.peek() {
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                if let Poll::Pending = future.as_mut().poll(context) {
                    println!("*** timer future pending");
                    *future_slot = Some(future);
                    break;
                } else {
                    println!("*** timer future ready");
                    self.heap.pop();
                }
            }
        }
    }
}

/// Timer Future.
pub struct TimerFuture {
    fire_at: Instant,
}

impl TimerFuture {

    /// Constructor.
    pub fn new(duration: Duration) -> Self {
        TimerFuture {
            fire_at: Instant::now() + duration,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let now = Instant::now();
        if self.fire_at <= now {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
