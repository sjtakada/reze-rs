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

use log::debug;
use log::error;

use eventum::core::*;
use eventum::uds_server::*;

use common::error::*;
use common::method::Method;

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

    /// Thread join handle.
    handle: JoinHandle<()>,

    /// Channel sender from Master To Protocol
    sender: mpsc::Sender<NexusToProto>,
}

/// Router Nexus.
pub struct RouterNexus {

    /// Event Manager.
    event_manager: Arc<Mutex<EventManager>>,

    /// 
    masters: Mutex<HashMap<ProtocolType, MasterTuple>>,

    /// Sender channel for ProtoToNexus.
    sender_p2n: Mutex<Option<mpsc::Sender<ProtoToNexus>>>,

    /// Sender channel for ProtoToZebra.
    sender_p2z: Mutex<Option<mpsc::Sender<ProtoToZebra>>>,

    /// UdsServer for Config.
    config_server: Mutex<Option<Arc<UdsServer>>>,

    /// UdsServer for Exec.
    exec_server: Mutex<Option<Arc<UdsServer>>>,
}

impl RouterNexus {

    /// Constructor.
    pub fn new(event_manager: Arc<Mutex<EventManager>>) -> RouterNexus {
        RouterNexus {
            event_manager: event_manager,
            masters: Mutex::new(HashMap::new()),
            sender_p2n: Mutex::new(None),
            sender_p2z: Mutex::new(None),
            config_server: Mutex::new(None),
            exec_server: Mutex::new(None),
        }
    }

    /// Return masters.
    fn get_sender(&self, p: &ProtocolType) -> Option<mpsc::Sender<NexusToProto>> {
        match self.masters.lock().unwrap().get(&p) {
            Some(tuple) => Some(tuple.sender.clone()),
            None => None,
        }
    }

    /// Set UdsServer for Config.
    pub fn set_config_server(&self, uds_server: Arc<UdsServer>) {
        self.config_server.lock().unwrap().replace(uds_server);
    }

    /// Set UdsServer for Exec.
    pub fn set_exec_server(&self, uds_server: Arc<UdsServer>) {
        self.exec_server.lock().unwrap().replace(uds_server);
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
        if let Some(tuple) = self.masters.lock().unwrap().remove(&proto) {
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
        if let Some(ref mut sender_p2n) = *self.sender_p2n.lock().unwrap() {
            return mpsc::Sender::clone(&sender_p2n);
        }
        panic!("failed to clone");
    }

    /// Clone ProtoToZebra mpsc::Sender.
    fn _clone_sender_p2z(&self) -> mpsc::Sender<ProtoToZebra> {
        if let Some(ref mut sender_p2z) = *self.sender_p2z.lock().unwrap() {
            return mpsc::Sender::clone(&sender_p2z)
        }
        panic!("failed to clone");
    }

    /// Entry point to start RouterNexus.
    pub fn start(nexus: Arc<RouterNexus>, event_manager: Arc<Mutex<EventManager>>) -> Result<(), CoreError> {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        nexus.sender_p2n.lock().unwrap().replace(sender_p2n);

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = nexus.spawn_zebra(nexus.clone_sender_p2n());
        nexus.sender_p2z.lock().unwrap().replace(sender_p2z);
        nexus.masters.lock().unwrap().insert(ProtocolType::Zebra, MasterTuple { handle, sender });


        // XXX spawn OSPF
        let (handle, sender, _sender_z2p) = nexus.spawn_protocol(ProtocolType::Ospf,
                                                               nexus.clone_sender_p2n(),
                                                               nexus._clone_sender_p2z());
        nexus.masters.lock().unwrap().insert(ProtocolType::Ospf, MasterTuple { handle, sender });

        // Register channel handler to event manager.
        let channel_handler = ProtoToNexusChannelHandler::new(nexus.clone(), receiver);
        event_manager.lock().unwrap().register_channel(Box::new(channel_handler));

        // Event loop.
        let runner = SimpleRunner::new();
        while !signal::is_sigint_caught() {
            let events = event_manager.lock().unwrap().poll();
            match runner.run(events) {
                Err(EventError::SystemShutdown) => break,
                _ => {}
            }
        }

        // Send termination message to all threads first.
        // TODO: is there better way to iterate hashmap and remove it at the same time?
        let mut v = Vec::new();
        for (proto, _tuple) in nexus.masters.lock().unwrap().iter_mut() {
            v.push(proto.clone());
        }

        for proto in &v {
            nexus.finish_protocol(proto);
        }

        // Nexus terminated.
        Err(CoreError::SystemShutdown)
    }
}

/// ProtoToNexus channel handler.
pub struct ProtoToNexusChannelHandler {

    /// RouterNexus.
    nexus: Arc<RouterNexus>,

    /// Receiver.
    receiver: mpsc::Receiver<ProtoToNexus>,
}

impl ProtoToNexusChannelHandler {

    /// Constructor.
    pub fn new(nexus: Arc<RouterNexus>,
               receiver: mpsc::Receiver<ProtoToNexus>
    ) -> ProtoToNexusChannelHandler {
        ProtoToNexusChannelHandler {
            nexus: nexus,
            receiver: receiver,
        }
    }
}

impl ChannelHandler for ProtoToNexusChannelHandler {

    fn poll_channel(&self) -> Vec<(EventType, Arc<dyn EventHandler>)> {
        let mut vec = Vec::new();

        while let Ok(message) = self.receiver.try_recv() {
            let handler = ProtoToNexusMessageHandler::new(self.nexus.clone(), message);

            vec.push((EventType::ChannelEvent, handler));
        }
        
        vec
    }
}

pub struct ProtoToNexusMessageHandler {
    nexus: Arc<RouterNexus>,
    message: ProtoToNexus,
}

impl ProtoToNexusMessageHandler {
    pub fn new(nexus: Arc<RouterNexus>, message: ProtoToNexus) -> Arc<dyn EventHandler> {
        Arc::new(ProtoToNexusMessageHandler {
            nexus: nexus,
            message: message,
        })
    }
}

unsafe impl Sync for ProtoToNexusMessageHandler {}
unsafe impl Send for ProtoToNexusMessageHandler {}

impl EventHandler for ProtoToNexusMessageHandler {

    /// Handle message.
    fn handle(&self, event_type: EventType) -> Result<(), EventError> {
        match event_type {
            EventType::ChannelEvent => match &self.message {
                ProtoToNexus::TimerRegistration((p, d, token)) => {
                    debug!("Received Timer Registration {} {}", p, token);

                    if let Some(tuple) = self.nexus.masters.lock().unwrap().get(&p) {
                        let entry = TimerEntry::new(*p, tuple.sender.clone(), *d, *token);
                        self.nexus.event_manager.lock().unwrap().register_timer(*d, Arc::new(entry));
                    }
                },
                ProtoToNexus::ConfigResponse((index, resp)) => {
                    if let Some(ref mut uds_server) = *self.nexus.config_server.lock().unwrap() {
                        let inner = uds_server.get_inner();
                        match inner.lookup_entry(*index) {
                            Some(entry) => {
                                let resp = match resp {
                                    Some(s) => s.clone(),//format!("{{\"status\":\"Error\",\"message\":\"{}\"}}", *s),
                                    None => Box::new(r#"{"status": "OK"}"#.to_string()),
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
                    if let Some(ref mut uds_server) = *self.nexus.exec_server.lock().unwrap() {
                        let inner = uds_server.get_inner();
                        match inner.lookup_entry(*index) {
                            Some(entry) => {
                                let resp = match resp {
                                    Some(s) => s.clone(),
                                    None => Box::new("{{}}".to_string()),
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
            },
            _ => assert!(false),
        }

        Ok(())
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
        let mds = Rc::new(MdsNode::new("NexusConfig"));

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
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), EventError> {
        match entry.stream_read() {
            Ok(request) => {
                match request_parse(request) {
                    Ok((method, path, body)) => {
                        debug!("Received request method: {}, path: {}, body: {:?}", method, path, body);

                        if let Err(err) = self.handle_request(entry.index(), method, &path, body) {
                            // Immediate error should send back error
                            let resp = err.json_status();
                            if let Err(_) = entry.stream_send(&resp) {
                                error!("Send in UdsServerHandler");
                            }

                            Err(EventError::UdsServerError(err.to_string()))
                        } else {
                            // Even if we get some response from handler, we don't send it right away.
                            Ok(())
                        }
                    },
                    Err(err) => Err(EventError::UdsServerError(err.to_string())),
                }
            }
            Err(err) => Err(err)
        }
    }

    /// Handle connect placeholder.
    fn handle_connect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), EventError> {
        debug!("handle_connect");
        Ok(())
    }

    /// Handle disconnect placeholder.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), EventError> {
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
    pub fn new(nexus: Arc<RouterNexus>) -> NexusExec {
        let mds = Rc::new(MdsNode::new("NexusExec"));

        let zebra_handler = Rc::new(MdsProtocolHandler::new_exec(ProtocolType::Zebra, nexus.clone()));
        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv4", zebra_handler.clone());
        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv6", zebra_handler.clone());
        MdsNode::register_handler(mds.clone(), "/exec/show/interface", zebra_handler.clone());

        NexusExec {
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
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), EventError> {
        match entry.stream_read() {
            Ok(request) => {
                match request_parse(request) {
                    Ok((method, path, body)) => {
                        debug!("Received request method: {}, path: {}, body: {:?}", method, path, body);

                        if let Err(err) = self.handle_request(entry.index(), method, &path, body) {
                            // Immediate error should send back error
                            let resp = err.json_status();
                            if let Err(_) = entry.stream_send(&resp) {
                                error!("Send in UdsServerHandler");
                            }

                            Err(EventError::UdsServerError(err.to_string()))
                        } else {
                            Ok(())
                        }
                    },
                    Err(err) => Err(EventError::UdsServerError(err.to_string())),
                }
            }
            Err(err) => Err(err)
        }
    }

    /// Handle connect placeholder.
    fn handle_connect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), EventError> {
        debug!("handle_connect");
        Ok(())
    }

    /// Handle disconnect placeholder.
    fn handle_disconnect(&self, server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), EventError> {
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
    fn handle(&self, e: EventType) -> Result<(), EventError> {
        match e {
            EventType::TimerEvent => {
                let sender = self.sender.lock().unwrap();

                if let Err(err) = sender.send(NexusToProto::TimerExpiration(self.token)) {
                    error!("Sending message to protocol {:?} {:?}", self.token, err);
                }
            },
            _ => {
                return Err(EventError::InvalidEvent);
            }
        }

        Ok(())
    }
}

/*
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
*/
