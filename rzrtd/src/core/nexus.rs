//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Router Nexus
//   Global container.
//   Initiate routing threads.
//   Handle messages from controller.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use log::debug;
use log::error;

use std::io;
use std::io::BufRead;
use std::io::Read;
use std::env;
//use std::fs::File;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
//use std::sync::Weak;
use std::boxed::Box;
use std::cell::RefCell;
use std::time::Duration;
//use std::time::Instant;

use quick_error::*;
use mio::*;
use mio::unix::EventedFd;
use mio_uds::UnixListener;
use mio_uds::UnixStream;

use super::event::*;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

use super::master::ProtocolMaster;
use crate::zebra::master::ZebraMaster;
use crate::bgp::master::BgpMaster;
use crate::ospf::master::OspfMasterInner;

use super::timer;

struct MasterTuple {
    // Thread Join handle
    handle: JoinHandle<()>,

    // Channel sender from Master To Protocol
    sender: mpsc::Sender<NexusToProto>,
}

pub struct RouterNexus {
    // MasterInner map
    masters: RefCell<HashMap<ProtocolType, MasterTuple>>,

    // Timer server
    timer_server: timer::Server,

    // Sender channel for ProtoToNexus
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    // Sender channel for ProtoToZebra
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,
}

impl UdsServerHandler for RouterNexus {
    // Process command.
    fn handle(&self, s: &str) -> Result<(), CoreError> {

        let command = s;

        match command {
            "ospf" => {
                // Spawn ospf instance
                let (handle, sender, sender_z2p) =
                    self.spawn_protocol(ProtocolType::Ospf,
                                        self.clone_sender_p2n(),
                                        self.clone_sender_p2z());
                self.masters.borrow_mut().insert(ProtocolType::Ospf, MasterTuple { handle, sender });

                // register sender_z2p to Zebra thread
            },
            "bgp" => {

            },
            "quit" => {
                return Err(CoreError::NexusTermination)
            }
            _ => {
                return Err(CoreError::CommandNotFound(command.to_string()))
            }
        }

        Ok(())
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum CoreError {
        NexusTermination {
            description("Nexus is terminated")
            display(r#"Nexus is terminated"#)
        }
        CommandNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
    }
}

impl RouterNexus {
    pub fn new() -> RouterNexus {
        RouterNexus {
            masters: RefCell::new(HashMap::new()),
            timer_server: timer::Server::new(),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let mut zebra = ZebraMaster::new();
            zebra.start(sender_p2n, receiver_n2p, receiver_p2z);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_p2z)
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2n: mpsc::Sender<ProtoToNexus>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ZebraToProto>) {
        // Create channel from Nexus to Protocol Master
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();

        // Create channel from Zebra To Protocol Master
        let (sender_z2p, receiver_z2p) = mpsc::channel::<ZebraToProto>();

        let handle = thread::spawn(move || {
            let protocol = Arc::new(ProtocolMaster::new(p));
            protocol.inner_set(
                match p {
                    ProtocolType::Ospf => Box::new(OspfMasterInner::new(protocol.clone())),
                    ProtocolType::Bgp => Box::new(BgpMaster::new(protocol.clone())),
                    _ => panic!("Not supported")
                });

            protocol.timers_set(timer::Client::new(protocol.clone()));
            protocol.start(sender_p2n, receiver_n2p, sender_p2z, receiver_z2p);
            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_z2p)
    }

    //
    fn finish_protocol(&mut self, proto: &ProtocolType) {
        if let Some(tuple) = self.masters.borrow_mut().remove(&proto) {
            tuple.sender.send(NexusToProto::ProtoTermination);

            match tuple.handle.join() {
                Ok(_ret) => {
                    debug!("protocol join succeeded");
                },
                Err(_err) => {
                    debug!("protocol join failed");
                }
            }
        }
    }

    fn clone_sender_p2n(&self) -> mpsc::Sender<ProtoToNexus> {
        if let Some(ref mut sender_p2n) = *self.sender_p2n.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2n);
        }
        panic!("failed to clone");
    }

    fn clone_sender_p2z(&self) -> mpsc::Sender<ProtoToZebra> {
        if let Some(ref mut sender_p2z) = *self.sender_p2z.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2z)
        }
        panic!("failed to clone");
    }

    pub fn message_callback(handler: Arc<UdsServerHandler>, entry: Arc<UdsServerEntry>) -> i32 {
        handler.handle("s");
        0
    }

    //
    pub fn start(&mut self) {
        // Create Unix Domain Socket to accept commands.
        let mut path = env::temp_dir();
        path.push("rzrtd.cli");

        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        self.sender_p2n.borrow_mut().replace(sender_p2n);

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = self.spawn_zebra(self.clone_sender_p2n());
        self.sender_p2z.borrow_mut().replace(sender_p2z);
        self.masters.borrow_mut().insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        // Read/Write FD Event Manager
        let event_manager = Arc::new(EventManager::new());
        let ms = UdsServer::start(event_manager.clone(), &path);

        ms.register_message_callback(Arc::new(RouterNexus::message_callback));

        'main: loop {
            event_manager.poll();

            // Process channels
            while let Ok(d) = receiver.try_recv() {
                match d {
                    ProtoToNexus::TimerRegistration((p, d, token)) => {
                        debug!("Received timer registration {} {}", p, token);

                        self.timer_server.register(p, d, token);
                    }
                    ProtoToNexus::ProtoException(s) => {
                        debug!("Received exception {}", s);
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));

            // Process timer
            match self.timer_server.pop_if_expired() {
                Some(entry) => {
                    match self.masters.borrow_mut().get(&entry.protocol) {
                        Some(tuple) => {
                            let result =
                                tuple.sender.send(NexusToProto::TimerExpiration(entry.token));
                            // TODO
                            match result {
                                Ok(_ret) => {},
                                Err(_err) => {}
                            }
                        }
                        None => {
                            panic!("Unexpected error");
                        }
                    }
                },
                None => { }
            }
        }

        // Send termination message to all threads first.
        // TODO: is there better way to iterate hashmap and remove it at the same time?
        let mut v = Vec::new();
        for (proto, _tuple) in self.masters.borrow_mut().iter_mut() {
            v.push(proto.clone());
        }

        for proto in &v {
            self.finish_protocol(proto);
        }

        // Nexus terminated.
    }
}


use std::path::PathBuf;

//
pub trait UdsServerHandler {
    fn handle(&self, s: &str) -> Result<(), CoreError>;
}

unsafe impl Send for UdsServerEntry {}
unsafe impl Sync for UdsServerEntry {}

//
pub type UdsServerCallback = FnMut(Arc<UdsServerHandler>, Arc<UdsServerEntry>) -> i32;

//
pub struct UdsServerEntry {
    stream: RefCell<Option<UnixStream>>,
}

impl UdsServerEntry {
    pub fn new() -> UdsServerEntry {
        UdsServerEntry {
            stream: RefCell::new(None),
        }
    }
}


impl EventHandler for UdsServerEntry {
    fn handle(&self, e: EventType, param: Option<Arc<EventParam>>) {
        match e {
            EventType::ReadEvent => {
                let mut buffer = String::new();
                if let Some(ref mut stream) = *self.stream.borrow_mut() {
                    stream.read_to_string(&mut buffer);

                    let command = buffer.trim();

                    debug!("received command {}", command);

/*
                    match self.process_command(&command) {
                        Err(CoreError::NexusTermination) => {
                            debug!("Termination");
                        },
                        Err(CoreError::CommandNotFound(str)) => {
                            error!("Command not found '{}'", str);
                        },
                        _ => {
                        }
                    }
*/
                }
            },
            _ => {
                debug!("Unknown event");
            }
        }
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

    // mio UnixListener
    listener: UnixListener,

    // Message Server Inner
    inner: RefCell<Option<Arc<UdsServerInner>>>,

    // Callbacks.
    message_callback: RefCell<Option<Arc<UdsServerCallback>>>,
}
  
impl UdsServer {
    fn new(event_manager: Arc<EventManager>, path: &PathBuf) -> UdsServer {
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(_) => panic!("UnixListener::bind() error"),
        };

        UdsServer {
            event_manager: RefCell::new(event_manager),
            listener: listener,
            inner: RefCell::new(None),
            message_callback: RefCell::new(None),
        }
    }

    pub fn start(mut event_manager: Arc<EventManager>, path: &PathBuf) -> Arc<UdsServer> {
        let server = Arc::new(UdsServer::new(event_manager.clone(), path));
        let inner = Arc::new(UdsServerInner::new(server.clone()));

        event_manager.register_read(&server.listener, inner.clone());

        server.inner.borrow_mut().replace(inner);
        server
    }

    pub fn register_connect_callback() {
    }

    pub fn register_disconnect_callback() {
    }

    pub fn register_message_callback(&self, callback: Arc<UdsServerCallback>) {
        self.message_callback.borrow_mut().replace(callback);
    }
}

impl EventHandler for UdsServerInner {
    fn handle(&self, e: EventType, param: Option<Arc<EventParam>>) {
        let mut server = self.server.borrow_mut();

        match e {
            EventType::ReadEvent => {
                match server.listener.accept() {
                    Ok(Some((stream, _addr))) => {
                        debug!("Got a message client: {:?}", _addr);

                        let entry = Arc::new(UdsServerEntry::new());
                        let event_manager = server.event_manager.borrow();

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
    }
}


            /*
            let mut events = Events::with_capacity(1024);
            poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();

            for event in events.iter() {
                match event.token() {
                    // ClI or API commands
                    Token(0) => {
                        let mut line = String::new();
                        io::stdin().lock().read_line(&mut line).unwrap();

                        let command = line.trim();

                        match self.process_command(&command) {
                            Err(CoreError::NexusTermination) => {
                                break 'main;
                            },
                            Err(CoreError::CommandNotFound(str)) => {
                                error!("Command not found '{}'", str);
                            },
                            _ => {
                            }
                        }
                    },
                    Token(1) => {
                        match listener.accept() {
                            Ok(Some((stream, _addr))) => {
                                println!("Got a client: {:?}", _addr);

                                poll.register(&stream, Token(2), Ready::readable(), PollOpt::edge()).unwrap();
                                token2evented.insert(2, RefCell::new(stream));
                            },
                            Ok(None) => println!("OK, but None???"),
                            Err(err) => println!("accept function failed: {:?}", err),
                        }
                    },
                    Token(2) => {
                        if let Some(stream) = token2evented.get(&2) {

                            let mut buffer = String::new();
                            let mut s = stream.borrow_mut();

                            s.read_to_string(&mut buffer);

                            let command = buffer.trim();

                            debug!("received command {}", command);

                            match self.process_command(&command) {
                                Err(CoreError::NexusTermination) => {
                                    break 'main;
                                },
                                Err(CoreError::CommandNotFound(str)) => {
                                    error!("Command not found '{}'", str);
                                },
                                _ => {
                                }
                            }
                        }
                    },
                    // fallback
                    _ => {
                        
                    }
                }
            }
             */
