//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Event Handler
// Fd Event manager
//

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use mio::*;
use mio::unix::EventedFd;

//
pub enum EventType {
    SimpleEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
}

//
pub enum EventParam {
    Param(String)
}

//
pub trait EventHandler {
    fn handle(&self, event_type: EventType, param: Option<Arc<EventParam>>);
}


//
pub struct EventManagerInner {
    // Token index.
    index: usize,

    // Token to handler map.
    handlers: HashMap<Token, Arc<EventHandler + Send + Sync>>,

    // mio::Poll
    poll: Poll,

    // poll timeout in msecs
    timeout: Duration,
}

pub struct EventManager {
    pub inner: RefCell<EventManagerInner>,
}

impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            inner: RefCell::new(EventManagerInner {
                index: 0,
                handlers: HashMap::new(),
                poll: Poll::new().unwrap(),
                timeout: Duration::from_millis(10),
            })
        }
    }

    pub fn register_read(&self, fd: &Evented, handler: Arc<EventHandler + Send + Sync>) {
        let mut inner = self.inner.borrow_mut();
        let token = Token(inner.index);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::readable(), PollOpt::level()).unwrap();

        inner.index += 1;
    }

    pub fn poll_get_events(&self) -> Events {
        let mut inner = self.inner.borrow_mut();
        let mut events = Events::with_capacity(1024);
        inner.poll.poll(&mut events, Some(inner.timeout));

        events
    }

    pub fn poll_get_handler(&self, event: Event) -> Option<Arc<EventHandler + Send + Sync>> {
        let mut inner = self.inner.borrow_mut();
        match inner.handlers.get(&event.token()) {
            Some(handler) => Some(handler.clone()),
            None => None,
        }
    }

    pub fn poll(&self) {
        let events = self.poll_get_events();

        for event in events.iter() {
            if let Some(handler) = self.poll_get_handler(event) {
                handler.handle(EventType::ReadEvent, None);
            }
        }
    }
}

