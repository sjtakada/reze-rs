//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Unix Domain Socket Server
//

use log::debug;

use std::io::Read;

use std::sync::Arc;
use std::cell::RefCell;
use std::path::PathBuf;

use mio_uds::UnixListener;
use mio_uds::UnixStream;

use super::error::*;
use super::event::*;

// Trait UdsServer callbacks.
pub trait UdsServerHandler {
    // callback when server accepts client connection.
    fn handle_connect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;

    // callback when server detects client disconnected.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;

    // callback when server entry received message.
    fn handle_message(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError>;
}

unsafe impl Send for UdsServerEntry {}
unsafe impl Sync for UdsServerEntry {}

// Unix Domain Socket server entry, created per connect.
pub struct UdsServerEntry {
    // Pointer to UdsServer.
    server: RefCell<Arc<UdsServer>>,

    // mio UnixStream.
    stream: RefCell<Option<UnixStream>>,
}

impl UdsServerEntry {
    pub fn new(server: Arc<UdsServer>) -> Arc<UdsServerEntry> {
        Arc::new(UdsServerEntry { server: RefCell::new(server),
                                  stream: RefCell::new(None) })
    }

    pub fn stream_read(&self) -> Option<String> {
        match *self.stream.borrow_mut() {
            Some(ref mut stream) => {
                let mut buffer = String::new();

                match stream.read_to_string(&mut buffer) {
                    Ok(_) => {},
                    Err(_) => {},
                }
                let command = String::from(buffer.trim());
                Some(command)
            },
            None => None
        }
    }
}


impl EventHandler for UdsServerEntry {
    fn handle(&self, e: EventType, _param: Option<Arc<EventParam>>) -> Result<(), CoreError> {
        match e {
            EventType::ReadEvent => {
                let server = self.server.borrow_mut();
                let handler = server.handler.borrow_mut();

                // Dispatch message to Server message handler.
                return handler.handle_message(server.clone(), self);
            },
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }
}

struct UdsServerInner {
    server: RefCell<Arc<UdsServer>>,
}

impl UdsServerInner {
    pub fn new(server: Arc<UdsServer>) -> UdsServerInner {
        UdsServerInner {
            server: RefCell::new(server)
        }
    }
}

unsafe impl Send for UdsServerInner {}
unsafe impl Sync for UdsServerInner {}

pub struct UdsServer {
    // EventManager
    event_manager: RefCell<Arc<EventManager>>,

    // Message Server Handler
    handler: RefCell<Arc<dyn UdsServerHandler>>,

    // Message Server Inner
    inner: RefCell<Option<Arc<UdsServerInner>>>,

    // mio UnixListener
    listener: UnixListener,
}
  
impl UdsServer {
    fn new(event_manager: Arc<EventManager>, handler: Arc<dyn UdsServerHandler>, path: &PathBuf) -> UdsServer {
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(_) => panic!("UnixListener::bind() error"),
        };

        UdsServer {
            event_manager: RefCell::new(event_manager),
            handler: RefCell::new(handler),
            inner: RefCell::new(None),
            listener: listener,
        }
    }

    pub fn start(event_manager: Arc<EventManager>, handler: Arc<dyn UdsServerHandler>, path: &PathBuf) -> Arc<UdsServer> {
        let server = Arc::new(UdsServer::new(event_manager.clone(), handler, path));
        let inner = Arc::new(UdsServerInner::new(server.clone()));

        event_manager.register_listen(&server.listener, inner.clone());

        server.inner.borrow_mut().replace(inner);
        server
    }
}

impl EventHandler for UdsServerInner {
    fn handle(&self, e: EventType, _param: Option<Arc<EventParam>>) -> Result<(), CoreError> {
        let server = self.server.borrow_mut();

        match e {
            EventType::ReadEvent => {
                match server.listener.accept() {
                    Ok(Some((stream, _addr))) => {
                        debug!("Got a message client: {:?}", _addr);

                        let entry = UdsServerEntry::new(server.clone());
                        let event_manager = server.event_manager.borrow();

                        if let Err(_) = server.handler.borrow_mut().handle_connect(server.clone(), &entry) {
                            //
                        }

                        event_manager.register_read(&stream, entry.clone());
                        entry.stream.borrow_mut().replace(stream);
                    },
                    Ok(None) => debug!("OK, but None???"),
                    Err(err) => debug!("accept function failed: {:?}", err),
                }
            },
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }
}
