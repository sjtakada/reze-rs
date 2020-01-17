//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Unix Domain Socket Client.
//

//use std::io::Read;
use std::sync::Arc;
//use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
//use std::net::Shutdown;

use log::{debug, error};
use mio_uds::UnixStream;

use super::error::*;
use super::event::*;

/// Trait UdsClient handler.
pub trait UdsClientHandler {

    /// callback when client connects to server.
    fn handle_connect(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), CoreError>;

    /// callback when client detects server disconnected.
    fn handle_disconnect(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), CoreError>;

    /// callback when client received message.
    fn handle_message(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), CoreError>;
}

unsafe impl Send for UdsClient {}
unsafe impl Sync for UdsClient {}

/// Unix Domain Socket client entry, created per connect.
pub struct UdsClient {

    /// mio UnixStream.
    stream: RefCell<Option<UnixStream>>,

    /// UdsClient handler.
    handler: RefCell<Arc<dyn UdsClientHandler>>,
}

/// UdsClient implementation.
impl UdsClient {

    /// Constructor.
    pub fn new(handler: Arc<dyn UdsClientHandler>) -> UdsClient {
        UdsClient {
            stream: RefCell::new(None),
            handler: RefCell::new(handler),
        }
    }

    /// Start connecting to server.
    pub fn start(event_manager: Arc<EventManager>,
                 handler: Arc<dyn UdsClientHandler>, path: &PathBuf) -> Result<Arc<UdsClient>, CoreError> {
        let client = Arc::new(UdsClient::new(handler));
        match UnixStream::connect(path) {
            Ok(stream) => {
                event_manager.register_read(&stream, client.clone());
                client.stream.borrow_mut().replace(stream);
                let _ = client.handler.borrow_mut().handle_connect(&client);
            },
            Err(err) => {
                error!("Error connecting to server {:?}", err);
                return Err(CoreError::UdsConnectError);
            }
        }

        Ok(client)
    }
}

/// EventHandler implementation for UdsClient.
impl EventHandler for UdsClient {

    /// Handle event.
    fn handle(&self, e: EventType) -> Result<(), CoreError> {
        match e {
            EventType::ReadEvent => {
                let handler = self.handler.borrow_mut();

                // Dispatch message to Server message handler.
                return handler.handle_message(self);
            },
            EventType::ErrorEvent => {
                let handler = self.handler.borrow_mut();

                // Dispatch message to Server message handler.
                return handler.handle_disconnect(self);
            },
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }
}
