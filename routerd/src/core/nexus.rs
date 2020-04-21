//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Router Nexus
//   Global container.
//   Initiate routing threads.
//   Handle messages from controller.
//   Dispatch commands to each protocol.
//   Run event manger to handle async events.
//

use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::boxed::Box;
use std::cell::Cell;
use std::cell::RefCell;
use std::time::Instant;
use std::time::Duration;

use std::env;
use std::os::unix::net::UnixListener;
use std::os::unix::io::AsRawFd;

use log::debug;
use log::error;

use common::consts::*;
use common::event::*;
use common::epoll::*;
use common::timer::*;
use common::error::*;
use common::method::Method;
use common::uds_server::*;

use super::signal;
use super::timer;
use super::utils::*;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;
use super::master::ProtocolMaster;
use super::mds::*;

use crate::zebra::master::ZebraMaster;
use crate::bgp::master::BgpMaster;
use crate::ospf::master::OspfMasterInner;


/// Thread handle and Channel tuple.
struct MasterTuple {

    /// Thread Join handle.
    handle: JoinHandle<()>,

    /// Channel sender from Master To Protocol
    sender: mpsc::Sender<NexusToProto>,
}

/// Router Nexus.
pub struct RouterNexus {

    /// MasterInner map.
    masters: RefCell<HashMap<ProtocolType, MasterTuple>>,

    /// Sender channel for ProtoToNexus.
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    /// Sender channel for ProtoToZebra.
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,

    /// UdsServer for Config.
    config_server: RefCell<Option<Arc<UdsServer>>>,

    /// NexusExec.
    exec_server: RefCell<Option<Arc<UdsServer>>>,
}

/// RouterNexus implementation.
impl RouterNexus {

    /// Constructor.
    pub fn new() -> RouterNexus {
        RouterNexus {
            masters: RefCell::new(HashMap::new()),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
            config_server: RefCell::new(None),
            exec_server: RefCell::new(None),
        }
    }

    /// Return masters.
    fn get_sender(&self, p: &ProtocolType) -> Option<mpsc::Sender<NexusToProto>> {
        match self.masters.borrow().get(&p) {
            Some(tuple) => Some(tuple.sender.clone()),
            None => None,
        }
    }

    /// Set UdsServer for Config.
    pub fn set_config_server(&self, uds_server: Arc<UdsServer>) {
        self.config_server.borrow_mut().replace(uds_server);
    }

    /// Set UdsServer for Exec.
    pub fn set_exec_server(&self, uds_server: Arc<UdsServer>) {
        self.exec_server.borrow_mut().replace(uds_server);
    }

    /// Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {

        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let zebra = Rc::new(ZebraMaster::new());
            ZebraMaster::init(zebra.clone());
            zebra.start(sender_p2n, receiver_n2p, receiver_p2z);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_p2z)
    }

    /// Construct MasterInner instance and spawn a thread.
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

    /// Shutdown and cleanup protocol gracefully.
    fn finish_protocol(&self, proto: &ProtocolType) {
        if let Some(tuple) = self.masters.borrow_mut().remove(&proto) {
            if let Err(err) = tuple.sender.send(NexusToProto::ProtoTermination) {
                error!("Send protocol termination {:?}", err);
            }

            match tuple.handle.join() {
                Ok(_ret) => {
                    debug!("protocol join succeeded");
                },
                Err(err) => {
                    error!("protocol join failed {:?}", err);
                }
            }
        }
    }

    /// Clone ProtoToNexus mpsc::Sender.
    fn clone_sender_p2n(&self) -> mpsc::Sender<ProtoToNexus> {
        if let Some(ref mut sender_p2n) = *self.sender_p2n.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2n);
        }
        panic!("failed to clone");
    }

    /// Clone ProtoToZebra mpsc::Sender.
    fn _clone_sender_p2z(&self) -> mpsc::Sender<ProtoToZebra> {
        if let Some(ref mut sender_p2z) = *self.sender_p2z.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2z)
        }
        panic!("failed to clone");
    }

    /// Entry point to start RouterNexus.
    pub fn start(nexus: Arc<RouterNexus>, event_manager: Arc<EventManager>,
                 future_manager: Arc<FutureManager>) -> Result<(), CoreError> {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        nexus.sender_p2n.borrow_mut().replace(sender_p2n);

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = nexus.spawn_zebra(nexus.clone_sender_p2n());
        nexus.sender_p2z.borrow_mut().replace(sender_p2z);
        nexus.masters.borrow_mut().insert(ProtocolType::Zebra, MasterTuple { handle, sender });


        let mut config_uds_path = env::temp_dir();
        config_uds_path.push(ROUTERD_CONFIG_UDS_FILENAME);
        let listener = UnixListener::bind(config_uds_path).unwrap();
        let raw_fd = listener.as_raw_fd();
        let future_manager_clone = future_manager.clone();

        // XXX
        future_manager.clone().register_read(listener.as_raw_fd(), async move {
            println!("** register read future");
            EpollFuture::new(future_manager_clone, raw_fd).await;
        });

        // XXX spawn OSPF
        let (handle, sender, _sender_z2p) = nexus.spawn_protocol(ProtocolType::Ospf,
                                                               nexus.clone_sender_p2n(),
                                                               nexus._clone_sender_p2z());
        nexus.masters.borrow_mut().insert(ProtocolType::Ospf, MasterTuple { handle, sender });

        // Register channel handler to event manager.
        let nexus_clone = nexus.clone();
        let handler = move |event_manager: &EventManager| -> Result<(), CoreError> {
            nexus_clone.handle_nexus_message(&receiver, event_manager);
            Ok(())
        };
        event_manager.set_channel_handler(Box::new(handler));

        // XXX
        future_manager.register_timer(Duration::from_secs(5), Box::new(|| {
            println!("*** timer fired {}");
        }));

        // Main event loop.
        while !signal::is_sigint_caught() {
            match event_manager.run() {
                Err(CoreError::SystemShutdown) => break,
                _ => {
                }
            }

            match future_manager.run() {
                Err(CoreError::SystemShutdown) => break,
                _ => {
                }
            }
        }

        // Send termination message to all threads first.
        // TODO: is there better way to iterate hashmap and remove it at the same time?
        let mut v = Vec::new();
        for (proto, _tuple) in nexus.masters.borrow_mut().iter_mut() {
            v.push(proto.clone());
        }

        for proto in &v {
            nexus.finish_protocol(proto);
        }

        // Nexus terminated.
        Err(CoreError::SystemShutdown)
    }

    /// Handle ProtoToNexus channel messsages.
    fn handle_nexus_message(&self, receiver: &mpsc::Receiver<ProtoToNexus>,
                            event_manager: &EventManager) {
        while let Ok(d) = receiver.try_recv() {
            match d {
                ProtoToNexus::TimerRegistration((p, d, token)) => {
                    debug!("Received Timer Registration {} {}", p, token);

                    if let Some(tuple) = self.masters.borrow_mut().get(&p) {
                        let entry = TimerEntry::new(p, tuple.sender.clone(), d, token);
                        event_manager.register_timer(d, Arc::new(entry));
                    }
                },
                ProtoToNexus::ConfigResponse((index, resp)) => {
                    if let Some(ref mut uds_server) = *self.config_server.borrow_mut() {
                        let inner = uds_server.get_inner();
                        match inner.lookup_entry(index) {
                            Some(entry) => {
                                let resp = match resp {
                                    Some(s) => format!("{{\"status\":\"Error\",\"message\":\"{}\"}}", *s),
                                    None => r#"{"status": "OK"}"#.to_string(),
                                };

                                if let Err(_err) = entry.stream_send(&resp) {
                                    error!("Send UdsServerEntry");
                                }
                            },
                            None => {
                                error!("No UdsServerEntry");
                            }
                        }
                    }
                },
                ProtoToNexus::ExecResponse((index, resp)) => {
                    if let Some(ref mut uds_server) = *self.exec_server.borrow_mut() {
                        let inner = uds_server.get_inner();
                        match inner.lookup_entry(index) {
                            Some(entry) => {
                                let resp = match resp {
                                    Some(s) => *s,
                                    None => "".to_string(),
                                };

                                if let Err(_err) = entry.stream_send(&resp) {
                                    error!("Send UdsServerEntry");
                                }
                            },
                            None => {
                                error!("No UdsServerEntry");
                            }
                        }
                    }
                },
                ProtoToNexus::ProtoException(s) => {
                    debug!("Received Exception {}", s);
                },
            }
        }
    }
}

/// Dispatch request to protocol.
pub struct MdsProtocolHandler
{
    /// Protocol Type.
    proto: ProtocolType,

    /// Nexus.
    nexus: RefCell<Arc<RouterNexus>>,

    /// Encoder.
    encoder: &'static dyn Fn(u32, Method, &str, Option<Box<String>>) -> NexusToProto
}

/// MdsProtocolHandler implementation.
impl MdsProtocolHandler {

    /// Constructor.
    pub fn new(proto: ProtocolType, nexus: Arc<RouterNexus>) -> MdsProtocolHandler {
        MdsProtocolHandler {
            proto: proto,
            nexus: RefCell::new(nexus),
            encoder: &|id: u32, method: Method, path: &str, body: Option<Box<String>>| -> NexusToProto {
                NexusToProto::ConfigRequest((id, method, path.to_string(), body))
            },
        }
    }

    /// Constructor.
    pub fn new_exec(proto: ProtocolType, nexus: Arc<RouterNexus>) -> MdsProtocolHandler {
        MdsProtocolHandler {
            proto: proto,
            nexus: RefCell::new(nexus),
            encoder: &|id: u32, method: Method, path: &str, body: Option<Box<String>>| -> NexusToProto {
                NexusToProto::ExecRequest((id, method, path.to_string(), body))
            },
        }
    }
}


/// MdsHandler implementation for MdsProtocolHandler.
impl MdsHandler for MdsProtocolHandler {

    /// Handle all methods.
    fn handle_generic(&self, id: u32, method: Method,
                      path: &str, params: Option<Box<String>>) -> Result<Option<String>, CoreError> {
        let nexus = self.nexus.borrow();

        match nexus.get_sender(&self.proto) {
            Some(sender) => {
                if let Err(_) = sender.send((*self.encoder)(id, method, path, params)) {
                    Err(CoreError::ChannelSendError(format!("{} {}", method, path)))
                } else {
                    Ok(None)
                }
            }
            None => {
                Err(CoreError::ChannelNoSender)
            }
        }
    }

    /// Return handle_generic implmented.
    fn is_generic(&self) -> bool {
        true
    }
}

/// NexusConfig
pub struct NexusConfig {

    /// MdsNode root.
    mds: RefCell<Rc<MdsNode>>,

    /// RouterNexus.
    _nexus: RefCell<Arc<RouterNexus>>,
}

/// NexusConfig implementation.
impl NexusConfig {

    /// Constructor.
    pub fn new(nexus: Arc<RouterNexus>) -> NexusConfig {
        let mds = Rc::new(MdsNode::new());

        let zebra_handler = Rc::new(MdsProtocolHandler::new(ProtocolType::Zebra, nexus.clone()));
        MdsNode::register_handler(mds.clone(), "/config/route_ipv4", zebra_handler.clone());
        MdsNode::register_handler(mds.clone(), "/config/route_ipv6", zebra_handler.clone());

        NexusConfig {
            mds: RefCell::new(mds),
            _nexus: RefCell::new(nexus),
        }
    }

    /// Dispatch request to MDS tree.
    fn handle_request(&self, id: u32, method: Method,
                      path: &str, body: Option<String>) -> Result<Option<String>, CoreError> {

        let body = match body {
            Some(s) => Some(Box::new(s)),
            None => None
        };

        let mds_root = self.mds.borrow().clone();

        MdsNode::handle(mds_root, id, method, path, body)
    }
}

/// UdsServerHandler implementation for NexusConfig.
impl UdsServerHandler for NexusConfig {

    /// Process request.
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        if let Some(request) = entry.stream_read() {
            match request_parse(request) {
                Ok((method, path, body)) => {
                    debug!("Received request method: {}, path: {}, body: {:?}", method, path, body);

                    if let Err(err) = self.handle_request(entry.index(), method, &path, body) {
                        Err(err)
                    } else {
                        // Even if we get some response from handler, we don't send it right away.
                        Ok(())
                    }
                },
                Err(err) => Err(err),
            }
        } else {
            Err(CoreError::RequestInvalid("(no message)".to_string()))
        }
    }

    /// Handle connect placeholder.
    fn handle_connect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), CoreError> {
        debug!("handle_connect");
        Ok(())
    }

    /// Handle disconnect placeholder.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        server.shutdown_entry(entry);

        debug!("handle_disconnect");
        Ok(())
    }
}

/// NexusExec.
pub struct NexusExec {

    /// MdsNode root.
    mds: RefCell<Rc<MdsNode>>,

    /// RouterNexus.
    _nexus: RefCell<Arc<RouterNexus>>,
}

/// NexusExec implementation.
impl NexusExec {

    /// Constructor.
    pub fn new(nexus: Arc<RouterNexus>) -> NexusConfig {
        let mds = Rc::new(MdsNode::new());

        let zebra_handler = Rc::new(MdsProtocolHandler::new_exec(ProtocolType::Zebra, nexus.clone()));
        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv4", zebra_handler.clone());
        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv6", zebra_handler.clone());

        NexusConfig {
            mds: RefCell::new(mds),
            _nexus: RefCell::new(nexus),
        }
    }

    /// Dispatch request to MDS tree.
    fn handle_request(&self, id: u32, method: Method,
                      path: &str, body: Option<String>) -> Result<Option<String>, CoreError> {

        let body = match body {
            Some(s) => Some(Box::new(s)),
            None => None
        };

        let mds_root = self.mds.borrow().clone();

        MdsNode::handle(mds_root, id, method, path, body)
    }
}

/// UdsServerHandler implementation for NexusExec.
impl UdsServerHandler for NexusExec {

    /// Process command.
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        if let Some(request) = entry.stream_read() {
            match request_parse(request) {
                Ok((method, path, body)) => {
                    debug!("Received request method: {}, path: {}, body: {:?}", method, path, body);

                    if let Err(err) = self.handle_request(entry.index(), method, &path, body) {
                        Err(err)
                    } else {
                        Ok(())
                    }
                },
                Err(err) => Err(err),
            }
        } else {
            Err(CoreError::RequestInvalid("(no message)".to_string()))
        }
    }

    /// Handle connect placeholder.
    fn handle_connect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), CoreError> {
        debug!("handle_connect");
        Ok(())
    }

    /// Handle disconnect placeholder.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        server.shutdown_entry(entry);

        debug!("handle_disconnect");
        Ok(())
    }
}


/// Timer entry.
pub struct TimerEntry
where Self: Send,
      Self: Sync
{
    pub sender: Mutex<mpsc::Sender<NexusToProto>>,
    pub protocol: ProtocolType,
    pub expiration: Mutex<Cell<Instant>>,
    pub token: u32,
}

/// Timer entry implementation.
impl TimerEntry {

    /// Constructor.
    pub fn new(p: ProtocolType, sender: mpsc::Sender<NexusToProto>, d: Duration, token: u32) -> TimerEntry {
        TimerEntry {
            sender: Mutex::new(sender),
            protocol: p,
            expiration: Mutex::new(Cell::new(Instant::now() + d)),
            token: token,
        }
    }
}

/// EventHandler implementation for TimerEntry.
impl EventHandler for TimerEntry {

    /// Event handler.
    fn handle(&self, e: EventType) -> Result<(), CoreError> {
        match e {
            EventType::TimerEvent => {
                let sender = self.sender.lock().unwrap();

                if let Err(err) = sender.send(NexusToProto::TimerExpiration(self.token)) {
                    error!("Sending message to protocol {:?} {:?}", self.token, err);
                }
            },
            _ => {
                error!("Unknown event");
            }
        }

        Ok(())
    }
}

/// TimerHandler implementation for TimerEntry.
impl TimerHandler for TimerEntry {

    /// Get expiration.
    fn expiration(&self) -> Instant {
        self.expiration.lock().unwrap().get()
    }

    /// Set expiration.
    fn set_expiration(&self, d: Duration) {
        self.expiration.lock().unwrap().set(Instant::now() + d);
    }
}

