//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Unix Domain Socket Client.
//

use std::io::Write;
use std::sync::Arc;
use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Instant;
use std::time::Duration;
//use std::net::Shutdown;

use log::{debug, error};
use mio_uds::UnixStream;

use super::error::*;
use super::event::*;
use super::timer::*;

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

    /// UdsClient Inner.
    inner: RefCell<Option<Arc<UdsClientInner>>>,
}

/// UdsClient implementation.
impl UdsClient {

    /// Constructor.
    pub fn new() -> UdsClient {
        UdsClient {
            inner: RefCell::new(None),
        }
    }

    /// Return UdsClientInner.
    pub fn get_inner(&self) -> Arc<UdsClientInner> {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            return inner.clone()
        }

        // should not happen.
        panic!("No inner exists");
    }

    /// Start UdsClient.
    pub fn start(event_manager: Arc<EventManager>,
                 handler: Arc<dyn UdsClientHandler>, path: &PathBuf) -> Arc<UdsClient> {

        let client = Arc::new(UdsClient::new());
        let inner = Arc::new(UdsClientInner::new(client.clone(), event_manager.clone(),
                                                 handler.clone(), path));

        client.inner.borrow_mut().replace(inner);
        client
    }

    /// Connect to server.
    pub fn connect(&self) {
        let inner = self.get_inner();
        let event_manager = inner.get_event_manager();

        match inner.connect() {
            Ok(_) => {
                if let Some(ref mut stream) = *inner.stream.borrow_mut() {
                    event_manager.register_read(stream, inner.clone());
                }
            },
            Err(_) => {
                let d = Duration::from_secs(5);
                event_manager.register_timer(d, inner.clone());
            },
        }
    }

    /// Send message.
    pub fn stream_send(&self, message: &str) {
        self.get_inner().stream_send(message);
    }
}

unsafe impl Send for UdsClientInner {}
unsafe impl Sync for UdsClientInner {}

/// UdsClient Inner.
pub struct UdsClientInner {

    /// Server path.
    path: PathBuf,
    
    /// UdsClient.
    client: RefCell<Arc<UdsClient>>,

    /// Event manager.
    event_manager: RefCell<Arc<EventManager>>,

    /// Message Client handler.
    handler: RefCell<Arc<dyn UdsClientHandler>>,

    /// Client stream.
    stream: RefCell<Option<UnixStream>>,

    /// Reconnect timer.
    reconnect: Cell<Instant>,
}

/// impl
impl UdsClientInner {

    ///
    pub fn new(client: Arc<UdsClient>, event_manager: Arc<EventManager>,
               handler: Arc<dyn UdsClientHandler>, path: &PathBuf) -> UdsClientInner {
        UdsClientInner {
            path: path.clone(),
            client: RefCell::new(client),
            event_manager: RefCell::new(event_manager),
            handler: RefCell::new(handler),
            stream: RefCell::new(None),
            reconnect: Cell::new(Instant::now()),
        }
    }

    pub fn connect(&self) -> Result<(), CoreError> {
        let client = self.client.borrow_mut();

        match UnixStream::connect(&self.path) {
            Ok(stream) => {
                self.stream.borrow_mut().replace(stream);
                let _ = self.handler.borrow_mut().handle_connect(&client);
                Ok(())
            },
            Err(err) => {
                error!("Error connecting to server {:?}", err);
                Err(CoreError::UdsConnectError)
            }
        }
    }

    ///
    pub fn get_event_manager(&self) -> Arc<EventManager> {
        self.event_manager.borrow_mut().clone()
    }

    pub fn stream_send(&self, message: &str) {
        match *self.stream.borrow_mut() {
            Some(ref mut stream) => {
                let _ = stream.write_all(message.as_bytes());
            },
            None => {
                error!("No stream");
            }
        }
    }
}

/// EventHandler implementation for UdsClient.
impl EventHandler for UdsClientInner {

    /// Handle event.
    fn handle(&self, e: EventType) -> Result<(), CoreError> {
        match e {
            EventType::TimerEvent => {
                // Reconnect timer expired.
                let client = self.client.borrow_mut().clone();
                client.connect();

                Ok(())
            },
            EventType::ReadEvent => {
                let handler = self.handler.borrow_mut();

                // Dispatch message to Server message handler.
                handler.handle_message(&self.client.borrow())
            },
            EventType::ErrorEvent => {
                self.stream.borrow_mut().take();

                let client = self.client.borrow_mut().clone();
                client.connect();

                let handler = self.handler.borrow_mut();

                // Dispatch message to Server message handler.
                handler.handle_disconnect(&self.client.borrow())
            },
            _ => {
                debug!("Unknown event");
                Err(CoreError::UnknownEvent)
            }
        }
    }
}

impl TimerHandler for UdsClientInner {

    fn expiration(&self) -> Instant {
        self.reconnect.get()
    }

    fn set_expiration(&self, d: Duration) {
        self.reconnect.set(Instant::now() + d);
    }
}
