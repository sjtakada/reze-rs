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

use super::error::*;

use mio::*;

/// Event types.
pub enum EventType {
    SimpleEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
    ErrorEvent,
}

/// Event parameters.
pub enum EventParam {
    Param(String)
}

/// Event Handler trait.
pub trait EventHandler {
    fn handle(&self, event_type: EventType, param: Option<Arc<EventParam>>) -> Result<(), CoreError>;

    fn set_token(&self, token: Token) {
        // Placeholder
    }

    fn get_token(&self) -> Token {
        // Placeholder
        Token(0)
    }
}

///
pub struct EventManagerInner {
    // Token index.
    index: usize,

    // Token to handler map.
    handlers: HashMap<Token, Arc<dyn EventHandler + Send + Sync>>,

    // mio::Poll
    poll: Poll,

    // poll timeout in msecs
    timeout: Duration,
}

///
pub struct EventManager {
    pub inner: RefCell<EventManagerInner>,
}

impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            inner: RefCell::new(EventManagerInner {
                index: 1,	// Reserve 0
                handlers: HashMap::new(),
                poll: Poll::new().unwrap(),
                timeout: Duration::from_millis(10),
            })
        }
    }

    pub fn register_listen(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut inner = self.inner.borrow_mut();
        let token = Token(inner.index);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::readable(), PollOpt::edge()).unwrap();

        // TODO: consider rollover?
        inner.index += 1;
    }

    pub fn register_read(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut inner = self.inner.borrow_mut();
        let token = Token(inner.index);

        handler.set_token(token);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::readable(), PollOpt::level()).unwrap();

        // TODO: consider rollover?
        inner.index += 1;
    }

    pub fn unregister_read(&self, fd: &dyn Evented, token: Token) {
        let mut inner = self.inner.borrow_mut();

        let e = inner.handlers.remove(&token);
        inner.poll.deregister(fd).unwrap();
    }

    pub fn poll_get_events(&self) -> Events {
        let inner = self.inner.borrow_mut();
        let mut events = Events::with_capacity(1024);

        match inner.poll.poll(&mut events, Some(inner.timeout)) {
            Ok(_) => {},
            Err(_) => {}
        }

        events
    }

    pub fn poll_get_handler(&self, event: Event) -> Option<Arc<dyn EventHandler + Send + Sync>> {
        let inner = self.inner.borrow_mut();
        match inner.handlers.get(&event.token()) {
            Some(handler) => Some(handler.clone()),
            None => None,
        }
    }

    pub fn poll(&self) -> Result<(), CoreError> {
        let events = self.poll_get_events();
        let mut terminated = false;

        for event in events.iter() {
            if let Some(handler) = self.poll_get_handler(event) {
                if event.readiness() == Ready::readable() {
                    match handler.handle(EventType::ReadEvent, None) {
                        Err(CoreError::NexusTermination) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
                else {
                    match handler.handle(EventType::ErrorEvent, None) {
                        Err(CoreError::NexusTermination) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
            }
        }

        if terminated {
            return Err(CoreError::NexusTermination);
        }

        Ok(())
    }
}

