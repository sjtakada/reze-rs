//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Event Handler
// Fd Event manager
//

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use mio::*;
use mio::unix::EventedFd;

pub enum EventType {
    SimpleEvent,
    ReadEvent,
    WriteEvent,
    TimerEvent,
}

pub enum EventParam {
    Param(String)
}

pub trait EventHandler {
    fn handle(&self, event_type: EventType, param: Option<Arc<EventParam>>);
}

pub struct FdEventManager {
    // Token index.
    index: usize,

    // Token to handler map.
    handlers: HashMap<Token, Arc<EventHandler + Send + Sync>>,

    // mio::Poll
    poll: Poll,

    // poll timeout in msecs
    timeout: Duration,
}

impl FdEventManager {
    pub fn new() -> FdEventManager {
        FdEventManager {
            index: 0,
            handlers: HashMap::new(),
            poll: Poll::new().unwrap(),
            timeout: Duration::from_millis(10),
        }
    }

    pub fn register_read(&mut self, fd: &EventedFd, handler: Arc<EventHandler + Send + Sync>) {
        let token = Token(self.index);

        self.handlers.insert(token, handler);
        self.poll.register(fd, token, Ready::readable(), PollOpt::level()).unwrap();

        self.index += 1;
    }

    pub fn poll(&mut self) {
        let mut events = Events::with_capacity(1024);
        self.poll.poll(&mut events, Some(self.timeout));

        for event in events.iter() {
            if let Some(handler) = self.handlers.get(&event.token()) {
                handler.handle(EventType::ReadEvent, None);
            }
        }
    }
}

