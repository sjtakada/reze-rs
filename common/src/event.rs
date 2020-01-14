//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Event Handler
// Fd Event manager
//

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use mio::*;

use super::error::*;

/// Event types.
pub enum EventType {
    SimpleEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
    ErrorEvent,
}

/// Event parameters -- TBD maybe not needed any more.
pub enum EventParam {
    String(String),
    Number(i64),
}

/// Event Handler trait.
/// Token is associated with EventHandler and certain event expected.
pub trait EventHandler {

    /// Handle event.
    fn handle(&self, event_type: EventType, param: Option<Arc<EventParam>>) -> Result<(), CoreError>;

    /// Set token to event handler.
    fn set_token(&self, _token: Token) {
        // Placeholder
    }

    /// Get token from event handler.
    fn get_token(&self) -> Token {
        // Placeholder
        Token(0)
    }
}

/// EventManager inner.
pub struct EventManagerInner {

    /// Token index.
    index: usize,

    /// Token to handler map.
    handlers: HashMap<Token, Arc<dyn EventHandler + Send + Sync>>,

    /// mio::Poll
    poll: Poll,

    /// poll timeout in msecs
    timeout: Duration,
}

/// Event Manager.
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
        let index = inner.index;
        let token = Token(index);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::readable(), PollOpt::edge()).unwrap();

        // TODO: consider index wrap around?
        inner.index += 1;
    }

    pub fn register_read(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut inner = self.inner.borrow_mut();
        let index = inner.index;
        let token = Token(index);

        handler.set_token(token);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::readable(), PollOpt::level()).unwrap();

        // TODO: consider index wrap around?
        inner.index += 1;
    }

    pub fn register_write(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut inner = self.inner.borrow_mut();
        let index = inner.index;
        let token = Token(index);

        handler.set_token(token);

        inner.handlers.insert(token, handler);
        inner.poll.register(fd, token, Ready::writable(), PollOpt::level()).unwrap();

        inner.index += 1;
    }

    pub fn unregister_read(&self, fd: &dyn Evented, token: Token) {
        let mut inner = self.inner.borrow_mut();

        let _e = inner.handlers.remove(&token);
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
                        Err(CoreError::SystemShutdown) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
                else if event.readiness() == Ready::writable() {
                    match handler.handle(EventType::WriteEvent, None) {
                        Err(CoreError::SystemShutdown) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
                else {
                    match handler.handle(EventType::ErrorEvent, None) {
                        Err(CoreError::SystemShutdown) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
            }
        }

        if terminated {
            return Err(CoreError::SystemShutdown);
        }

        Ok(())
    }
}

