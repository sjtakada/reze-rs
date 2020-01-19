//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Event Handler
//  Fd Event manager
//

use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use mio::*;

use super::error::*;
use super::timer::*;

/// Event types.
pub enum EventType {
    BasicEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
    ErrorEvent,
}

/// Event Handler trait.
/// Token is associated with EventHandler and certain event expected.
pub trait EventHandler
where Self: Send,
      Self: Sync
{
    /// Handle event.
    fn handle(&self, event_type: EventType) -> Result<(), CoreError>;

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

/// File Descriptor EventManager.
pub struct FdEvent {

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

    /// FD Events.
    fd_events: RefCell<FdEvent>,

    /// Timer Events.
    timers: RefCell<TimerServer>,
}

/// EventManager implementation.
impl EventManager {

    /// Constructor.
    pub fn new() -> EventManager {
        EventManager {
            fd_events: RefCell::new(FdEvent {
                index: 1,	// Reserve 0
                handlers: HashMap::new(),
                poll: Poll::new().unwrap(),
                timeout: Duration::from_millis(10),
            }),
            timers: RefCell::new(TimerServer::new()),
        }
    }

    /// Register listen socket.
    pub fn register_listen(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut fd_events = self.fd_events.borrow_mut();
        let index = fd_events.index;
        let token = Token(index);

        fd_events.handlers.insert(token, handler);
        fd_events.poll.register(fd, token, Ready::readable(), PollOpt::edge()).unwrap();

        // TODO: consider index wrap around?
        fd_events.index += 1;
    }

    /// Register read socket.
    pub fn register_read(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut fd_events = self.fd_events.borrow_mut();
        let index = fd_events.index;
        let token = Token(index);

        handler.set_token(token);

        fd_events.handlers.insert(token, handler);
        fd_events.poll.register(fd, token, Ready::readable(), PollOpt::level()).unwrap();

        // TODO: consider index wrap around?
        fd_events.index += 1;
    }

    /// Register write socket.
    pub fn register_write(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler + Send + Sync>) {
        let mut fd_events = self.fd_events.borrow_mut();
        let index = fd_events.index;
        let token = Token(index);

        handler.set_token(token);

        fd_events.handlers.insert(token, handler);
        fd_events.poll.register(fd, token, Ready::writable(), PollOpt::level()).unwrap();

        fd_events.index += 1;
    }

    /// Unregister read socket.
    pub fn unregister_read(&self, fd: &dyn Evented, token: Token) {
        let mut fd_events = self.fd_events.borrow_mut();

        let _e = fd_events.handlers.remove(&token);
        fd_events.poll.deregister(fd).unwrap();
    }

    /// Poll and return events ready to read or write.
    pub fn poll_get_events(&self) -> Events {
        let fd_events = self.fd_events.borrow_mut();
        let mut events = Events::with_capacity(1024);

        match fd_events.poll.poll(&mut events, Some(fd_events.timeout)) {
            Ok(_) => {},
            Err(_) => {}
        }

        events
    }

    /// Return handler associated with token.
    pub fn poll_get_handler(&self, event: Event) -> Option<Arc<dyn EventHandler + Send + Sync>> {
        let fd_events = self.fd_events.borrow_mut();
        match fd_events.handlers.get(&event.token()) {
            Some(handler) => Some(handler.clone()),
            None => None,
        }
    }

    /// Poll and handle events.
    pub fn poll_fd(&self) -> Result<(), CoreError> {
        let events = self.poll_get_events();
        let mut terminated = false;

        for event in events.iter() {
            if let Some(handler) = self.poll_get_handler(event) {
                if event.readiness() == Ready::readable() {
                    match handler.handle(EventType::ReadEvent) {
                        Err(CoreError::SystemShutdown) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
                else if event.readiness() == Ready::writable() {
                    match handler.handle(EventType::WriteEvent) {
                        Err(CoreError::SystemShutdown) => {
                            terminated = true
                        },
                        _ => {
                        }
                    }
                }
                else {
                    match handler.handle(EventType::ErrorEvent) {
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

//    /// Register timer.
//    pub fn register_timer(&self, d: Duration, mut handler: Rc<dyn TimerHandler>) {
//
//    }
}

