//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Unix Domain Socket Server.
//

use std::io::Read;
use std::sync::Arc;
use std::cell::Cell;
use std::cell::RefCell;
use std::path::PathBuf;
use std::net::Shutdown;

use log::{debug, error};
use mio::Token;
use mio_uds::UnixListener;
use mio_uds::UnixStream;

use super::error::*;
use super::event::*;

/// Trait UdsServer callbacks.
pub trait UdsServerHandler {

    /// callback when server accepts client connection.
    fn handle_connect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;

    /// callback when server detects client disconnected.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;

    /// callback when server entry received message.
    fn handle_message(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;
}

unsafe impl Send for UdsServerEntry {}
unsafe impl Sync for UdsServerEntry {}

/// Unix Domain Socket server entry, created per connect.
pub struct UdsServerEntry {

    /// EventHandler token.
    token: Cell<Token>,

    /// Pointer to UdsServer.
    server: RefCell<Arc<UdsServer>>,

    /// mio UnixStream.
    stream: RefCell<Option<UnixStream>>,
}

/// UdsServerEntry implementation.
impl UdsServerEntry {

    /// Constructor.
    pub fn new(server: Arc<UdsServer>) -> UdsServerEntry {
        UdsServerEntry {
            token: Cell::new(Token(0)),
            server: RefCell::new(server),
            stream: RefCell::new(None)
        }
    }

    /// Read stream and return String - TBD.
    pub fn stream_read(&self) -> Option<String> {
        match *self.stream.borrow_mut() {
            Some(ref mut stream) => {
                let mut buffer = String::new();

                match stream.read_to_string(&mut buffer) {
                    Ok(_) => {
                        // may not come here.
                    },
                    Err(err) => {
                        if err.kind() != std::io::ErrorKind::WouldBlock {
                            error!("Error: {}", err);
                            return None
                        }
                    },
                }

                let command = String::from(buffer.trim());
                Some(command)
            },
            None => None
        }
    }
}

/// Drop implementation.
impl Drop for UdsServerEntry {
    fn drop(&mut self) {
        debug!("Drop UdsServerEntry");
    }
}

/// EventHandler implementation for UdsServerEntry.
impl EventHandler for UdsServerEntry {

    /// Handle event.
    fn handle(&self, e: EventType) -> Result<(), CoreError> {
        match e {
            EventType::ReadEvent => {
                let server = self.server.borrow_mut();
                let inner = server.get_inner();
                let handler = inner.handler.borrow_mut();

                // Dispatch message to Server message handler.
                return handler.handle_message(server.clone(), self);
            },
            EventType::ErrorEvent => {
                let server = self.server.borrow_mut();
                let inner = server.get_inner();
                let handler = inner.handler.borrow_mut();

                // Dispatch message to Server message handler.
                return handler.handle_disconnect(server.clone(), self);
            },
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }

    /// Set token to entry.
    fn set_token(&self, token: Token) {
        self.token.replace(token);
    }

    /// Get token from entry.
    fn get_token(&self) -> Token {
        self.token.get()
    }
}

/// UdsServer Inner.
pub struct UdsServerInner {

    /// UdsServer
    server: RefCell<Arc<UdsServer>>,

    /// EventManager.
    event_manager: RefCell<Arc<EventManager>>,

    /// Message Server Handler.
    handler: RefCell<Arc<dyn UdsServerHandler>>,

    /// mio UnixListener.
    listener: UnixListener,
}

/// UdsServerInner implementation.
impl UdsServerInner {
    pub fn new(server: Arc<UdsServer>, event_manager: Arc<EventManager>,
               handler: Arc<dyn UdsServerHandler>, path: &PathBuf) -> UdsServerInner {
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(_) => panic!("UnixListener::bind() error"),
        };

        UdsServerInner {
            server: RefCell::new(server),
            event_manager: RefCell::new(event_manager),
            handler: RefCell::new(handler),
            listener: listener,
        }
    }
}

unsafe impl Send for UdsServerInner {}
unsafe impl Sync for UdsServerInner {}

/// UdsServer Message Server.
pub struct UdsServer {

    /// UdsServer Inner.
    inner: RefCell<Option<Arc<UdsServerInner>>>,
}
  
/// UdsServer implementation.
impl UdsServer {

    /// Constructor.
    fn new() -> UdsServer {
        UdsServer {
            inner: RefCell::new(None),
        }
    }

    /// Return UdsServerInner.
    pub fn get_inner(&self) -> Arc<UdsServerInner> {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            return inner.clone()
        }

        // should not happen.
        panic!();
    }

    /// Start UdsServer.
    pub fn start(event_manager: Arc<EventManager>, handler: Arc<dyn UdsServerHandler>, path: &PathBuf) -> Arc<UdsServer> {
        let server = Arc::new(UdsServer::new());
        let inner = Arc::new(UdsServerInner::new(server.clone(), event_manager.clone(), handler.clone(), path));

        event_manager.register_listen(&inner.listener, inner.clone());

        server.inner.borrow_mut().replace(inner);
        server
    }

    /// Shutdown UdsServerEntry.
    pub fn shutdown_entry(&self, entry: &UdsServerEntry) {
        if let Some(ref mut stream) = *entry.stream.borrow_mut() {
            self.get_inner().event_manager.borrow().unregister_read(stream, entry.get_token());

            if let Err(err) = stream.shutdown(Shutdown::Both) {
                error!("Entry shutdown error {}", err.to_string());
            }
        }

        entry.stream.replace(None);
    }
}

/// EventHandler implementation for UdsServerInner.
impl EventHandler for UdsServerInner {

    /// Event handler.
    fn handle(&self, e: EventType) -> Result<(), CoreError> {
        let server = self.server.borrow_mut();

        match e {
            EventType::ReadEvent => {
                match self.listener.accept() {
                    Ok(Some((stream, _addr))) => {
                        debug!("Accept a UDS client: {:?}", _addr);

                        let entry = Arc::new(UdsServerEntry::new(server.clone()));
                        let event_manager = self.event_manager.borrow();

                        if let Err(_) = self.handler.borrow_mut().handle_connect(server.clone(), &entry) {
                            error!("UDS Server handler error");
                        }

                        event_manager.register_read(&stream, entry.clone());
                        entry.stream.borrow_mut().replace(stream);
                    },
                    Ok(None) => debug!("OK, but None???"),
                    Err(err) => debug!("Accept failed: {:?}", err),
                }
            },
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }
}
