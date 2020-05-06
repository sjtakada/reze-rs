//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Event Handler
//  Fd Event manager
//

use std::thread;
use std::collections::HashMap;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use mio::*;
use log::error;
use log::debug;

use super::consts::*;
use super::error::*;
use super::timer::*;
use super::channel::*;

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
    handlers: HashMap<Token, Arc<dyn EventHandler>>,

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

    /// Channel handler function.
    channel_handler: RefCell<Option<Box<dyn Fn(&EventManager) -> Result<(), CoreError>>>>,

    /// Channel events.
    ch_events: RefCell<ChannelManager>,
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
                timeout: Duration::from_millis(EVENT_MANAGER_TICK),
            }),
            timers: RefCell::new(TimerServer::new()),
            channel_handler: RefCell::new(None),
            ch_events: RefCell::new(ChannelManager::new()),
        }
    }

    ///
    pub fn init_channel_manager(event_manager: Arc<EventManager>) {
        let event_manager_clone = event_manager.clone();

        event_manager.ch_events.borrow_mut().set_event_manager(event_manager_clone);
    }

    /// Register listen socket.
    pub fn register_listen(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler>) {
        let mut fd_events = self.fd_events.borrow_mut();
        let index = fd_events.index;
        let token = Token(index);

        fd_events.handlers.insert(token, handler);
        fd_events.poll.register(fd, token, Ready::readable(), PollOpt::edge()).unwrap();

        // TODO: consider index wrap around?
        fd_events.index += 1;
    }

    /// Register read socket.
    pub fn register_read(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler>) {
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
    pub fn register_write(&self, fd: &dyn Evented, handler: Arc<dyn EventHandler>) {
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
    pub fn poll_get_handler(&self, event: Event) -> Option<Arc<dyn EventHandler>> {
        let fd_events = self.fd_events.borrow_mut();
        match fd_events.handlers.get(&event.token()) {
            Some(handler) => Some(handler.clone()),
            None => None,
        }
    }

    /// Poll FDs and handle events.
    pub fn poll_fd(&self) -> Result<(), CoreError> {
        let events = self.poll_get_events();
        let mut terminated = false;

        for event in events.iter() {
            if let Some(handler) = self.poll_get_handler(event) {
                let result = if event.readiness() == Ready::readable() {
                    handler.handle(EventType::ReadEvent)
                } else if event.readiness() == Ready::writable() {
                    handler.handle(EventType::WriteEvent)
                } else {
                    handler.handle(EventType::ErrorEvent)
                };

                match result {
                    Err(CoreError::SystemShutdown) => {
                        terminated = true;
                    }
                    Err(err) => {
                        error!("Poll fd {:?}", err);
                    }
                    Ok(_) => {
                        debug!("Poll fd OK");
                    }
                }
            }
        }

        if terminated {
            Err(CoreError::SystemShutdown)
        } else {
            Ok(())
        }
    }

    /// Register timer.
    pub fn register_timer(&self, d: Duration, handler: Arc<dyn TimerHandler>) {
        let timers = self.timers.borrow();
        timers.register(d, handler);
    }

    /// Poll timers and handle events.
    pub fn poll_timer(&self) -> Result<(), CoreError> {
        while let Some(handler) = self.timers.borrow().run() {
            let result = handler.handle(EventType::TimerEvent);

            match result {
                Err(err) => {
                    error!("Poll timer {:?}", err);
                }
                _ => {

                }
            }
        }

        Ok(())
    }

    /// Set channel handler.
    pub fn set_channel_handler(&self, handler: Box<dyn Fn(&EventManager) -> Result<(), CoreError>>) {
        self.channel_handler.borrow_mut().replace(handler);
    }

    /// Poll channel handler.
    pub fn poll_channel(&self) -> Result<(), CoreError> {
        if let Some(ref mut handler) = *self.channel_handler.borrow_mut() {
            handler(self)
        } else {
            Ok(())
        }
    }

    /// Register channel handler.
    pub fn register_channel(&self, handler: Box<dyn ChannelHandler>) {
        self.ch_events.borrow_mut().register_handler(handler);
    }

    /// Poll channel handlers.
    pub fn poll_channel_events(&self) -> Result<(), CoreError> {
        loop {
            match self.ch_events.borrow_mut().poll_channel() {
                Ok(_) => println!("*** poll_channel_events"),
                Err(err) => return Err(err)
            }
        }
    }

    /// Sleep certain period to have other events to occur.
    pub fn sleep(&self) {
        // TBD: we should sleep MIN(earlist timer, Tick).
        thread::sleep(Duration::from_millis(EVENT_MANAGER_TICK));
    }

    /// Event loop, but just a single iteration of all possible events.
    pub fn run(&self) -> Result<(), CoreError> {
        // Process events.
        if let Err(err) = self.poll_fd() {
            return Err(err)
        }

        if let Err(err) = self.poll_channel_events() {
            return Err(err)
        }

        // Process timer.
        if let Err(err) = self.poll_timer() {
            return Err(err)
        }

        // Wait a little bit.
        self.sleep();

        Ok(())
    }
}

/// Utility to blocking until fd gets readable.
pub fn wait_until_readable(fd: &dyn Evented) {
    let poll = Poll::new().unwrap();
    poll.register(fd, Token(0), Ready::readable(), PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(1024);

    let _ = poll.poll(&mut events, None);
}

/// Utility to blocking until fd gets writable.
pub fn wait_until_writable(fd: &dyn Evented) {
    let poll = Poll::new().unwrap();
    poll.register(fd, Token(0), Ready::writable(), PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(1024);

    let _ = poll.poll(&mut events, None);
}
