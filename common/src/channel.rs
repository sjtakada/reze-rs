//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Channel Event Manager/Handler
//

use std::cell::RefCell;
use std::sync::mpsc;

use super::event::*;
use super::error::*;

/// Channel Manager.
pub struct ChannelManager
{
    handlers: RefCell<Vec<Box<dyn ChannelHandler>>>,
}

impl ChannelManager {

    /// Constructor.
    pub fn new() -> ChannelManager {
        ChannelManager {
            handlers: RefCell::new(Vec::new()),
        }
    }

    /// Register handler.
    pub fn register_handler(&self, handler: Box<ChannelHandler>) {
        self.handlers.borrow_mut().push(handler);
    }

    /// Poll
    pub fn poll_channel(&self) -> Result<(), CoreError> {
        Ok(())
    }
}

///
pub trait ChannelHandler {

}

/// Channel Handler trait.
pub trait ChannelMessageHandler<T>: ChannelHandler {
    fn handle_message(&self, event_manager: &EventManager,
                      receiver: &mpsc::Receiver<T>) -> Result<(), CoreError>;
}
