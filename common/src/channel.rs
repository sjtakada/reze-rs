//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Channel Event Manager/Handler
//

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc;

use super::event::*;
use super::error::*;

/// Channel Manager.
pub struct ChannelManager
{
    /// EventManager.
    event_manager: RefCell<Option<Arc<EventManager>>>,

    /// Channel Message Handlers.
    handlers: RefCell<Vec<Box<dyn ChannelHandler>>>,

    /// Round Robin index.
    index: usize,
}

impl ChannelManager {

    /// Constructor.
    pub fn new() -> ChannelManager {
        ChannelManager {
            event_manager: RefCell::new(None),
            handlers: RefCell::new(Vec::new()),
            index: 0usize,
        }
    }

    /// Set Event Manager.
    pub fn set_event_manager(&self, event_manager: Arc<EventManager>) {
        self.event_manager.borrow_mut().replace(event_manager);
    }

    /// Register handler.
    pub fn register_handler(&self, handler: Box<dyn ChannelHandler>) {
        self.handlers.borrow_mut().push(handler);
    }

    /// Poll
    pub fn poll_channel(&self) -> Result<(), CoreError> {
        if let Some(ref event_manager) = *self.event_manager.borrow() {
            let len = self.handlers.borrow().len();
            let handler = &self.handlers.borrow()[self.index % len];

            (*handler).handle_message(event_manager.clone())
        } else {
            Err(CoreError::ChannelQueueEmpty)
//            panic!("No Event manager")
        }
    }
}

/// Channel Handler trait.
pub trait ChannelHandler {

    /// Handle message.
    fn handle_message(&self, event_manager: Arc<EventManager>) -> Result<(), CoreError>;
}
