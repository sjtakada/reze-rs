//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Unix Domain Socket Client.
//

use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Instant;
use std::time::Duration;
use std::sync::Mutex;

//use std::net::Shutdown;

use log::error;
use mio::net::UnixStream;

use eventum::*;


/// Trait UdsClient handler.
pub trait UdsClientHandler {

    /// callback when client connects to server.
    fn handle_connect(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), EventError>;

    /// callback when client detects server disconnected.
    fn handle_disconnect(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), EventError>;

    /// callback when client received message.
    fn handle_message(&self, /*client: Arc<UdsClient>, */entry: &UdsClient) -> Result<(), EventError>;
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
    pub fn start(event_manager: Arc<Mutex<EventManager>>,
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
                    event_manager
                        .lock()
                        .unwrap()
                        .register_read_write(stream, inner.clone());
                }
            },
            Err(_) => self.connect_timer(),
        }
    }

    /// Start connect timer.
    pub fn connect_timer(&self) {
        let inner = self.get_inner();
        let event_manager = inner.get_event_manager();

        event_manager
            .lock()
            .unwrap()
            .register_timer(Duration::from_secs(5), inner.clone());
    }

    /// Send message.
    pub fn stream_send(&self, message: &str) -> Result<(), EventError> {
        self.get_inner().stream_send(message)
    }

    /// Receive message.
    pub fn stream_read(&self) -> Result<String, EventError> {
        self.get_inner().stream_read()
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
    event_manager: Arc<Mutex<EventManager>>,

    /// Message Client handler.
    handler: RefCell<Arc<dyn UdsClientHandler>>,

    /// Client stream.
    stream: RefCell<Option<UnixStream>>,

    /// Reconnect timer.
    reconnect: Cell<Instant>,
}

/// UdsClientInner implementation.
impl UdsClientInner {

    /// Constructdor.
    pub fn new(client: Arc<UdsClient>, event_manager: Arc<Mutex<EventManager>>,
               handler: Arc<dyn UdsClientHandler>, path: &PathBuf)
               -> UdsClientInner {
        UdsClientInner {
            path: path.clone(),
            client: RefCell::new(client),
            event_manager: event_manager,
            handler: RefCell::new(handler),
            stream: RefCell::new(None),
            reconnect: Cell::new(Instant::now()),
        }
    }

    /// Connect to server.
    pub fn connect(&self) -> Result<(), EventError> {
        let client = self.client.borrow_mut();

        match UnixStream::connect(&self.path) {
            Ok(stream) => {
                self.stream.borrow_mut().replace(stream);
                let _ = self.handler.borrow_mut().handle_connect(&client);
                Ok(())
            },
            Err(_) => {
                Err(EventError::UdsConnectError)
            }
        }
    }

    /// Return event manager.
    pub fn get_event_manager(&self) -> Arc<Mutex<EventManager>> {
        self.event_manager.clone()
    }

    /// Send a message through UnixStream.
    pub fn stream_send(&self, message: &str) -> Result<(), EventError> {
        match *self.stream.borrow_mut() {
            Some(ref mut stream) => {
                if let Err(_err) = stream.write_all(message.as_bytes()) {
                    return Err(EventError::UdsWriteError)
                }
            },
            None => {
                return Err(EventError::UdsWriteError)
            }
        }

        Ok(())
    }

    /// Receive a message through UnixStream.
    /// Return UTF8 string. If it is empty, consider an error.
    pub fn stream_read(&self) -> Result<String, EventError> {
        match *self.stream.borrow_mut() {
            Some(ref mut stream) => {
                let mut buffer = Vec::new();

                if let Err(err) = stream.read_to_end(&mut buffer) {
                    if err.kind() != std::io::ErrorKind::WouldBlock {
                        return Err(EventError::ReadError(err.to_string()))
                    }
                }

                let str = std::str::from_utf8(&buffer).unwrap();
                if str.len() > 0 {
                    let message = String::from(str);
                    Ok(message)
                } else {
                    Err(EventError::ReadError("Empty string from stream".to_string()))
                }
            },
            None => {
                Err(EventError::NoStream)
            }
        }
    }
}

/// EventHandler implementation for UdsClient.
impl EventHandler for UdsClientInner {

    /// Handle event.
    fn handle(&self, e: EventType) -> Result<(), EventError> {
        match e {
            EventType::TimerEvent => {
                // Reconnect timer expired.
                let client = self.client.borrow_mut().clone();
                client.connect();

                Ok(())
            },
            EventType::ReadEvent => {
                let handler = self.handler.borrow_mut();
                let client = self.client.borrow();

                // Dispatch message to message handler.
                if let Err(_) = handler.handle_message(&client) {
                    // If read causes an error, most likely server disconnected,
                    // so schedule reconnect.
                    client.connect_timer();
                }

                Ok(())
            },
            EventType::ErrorEvent => {
                self.stream.borrow_mut().take();

                // TODO: Schedule reconnect timer.
                let client = self.client.borrow();
                client.connect_timer();

                let handler = self.handler.borrow_mut();

                // Dispatch message to Client message handler.
                handler.handle_disconnect(&client)
            },
            _ => {
                Err(EventError::InvalidEvent)
            }
        }
    }
}

