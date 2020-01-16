//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Router Nexus
//   Global container.
//   Initiate routing threads.
//   Handle messages from controller.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::boxed::Box;
use std::cell::RefCell;
use std::str::FromStr;
use std::time::Instant;
use std::time::Duration;

use log::debug;
use log::error;

use common::event::*;
use common::timer::*;
use common::error::*;
use common::uds_server::*;

use super::signal;
use super::timer;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;
use super::master::ProtocolMaster;
use super::config::*;
use super::config_master::*;

use crate::zebra::master::ZebraMaster;
use crate::bgp::master::BgpMaster;
use crate::ospf::master::OspfMasterInner;


/// Thread handle and Channel tuple.
struct MasterTuple {

    /// Thread Join handle
    handle: JoinHandle<()>,

    /// Channel sender from Master To Protocol
    sender: mpsc::Sender<NexusToProto>,
}

/// Router Nexus.
pub struct RouterNexus {

    /// Global config.
    config: RefCell<ConfigMaster>,

    /// MasterInner map
    masters: RefCell<HashMap<ProtocolType, MasterTuple>>,

    /// Timer server
    timer_server: RefCell<TimerServer>,

    /// Sender channel for ProtoToNexus
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    /// Sender channel for ProtoToZebra
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,
}

/// UdsServerHandler implementation for RouterNexus.
impl UdsServerHandler for RouterNexus {

    /// Process command.
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        if let Some(command) = entry.stream_read() {
            let mut lines = command.lines();

            if let Some(req) = lines.next() {
                let mut words = req.split_ascii_whitespace();

                if let Some(method_str) = words.next() {
                    if let Ok(method) = Method::from_str(method_str) {

                        if let Some(path) = words.next() {
                            let mut body: Option<String> = None;

                            // Skip a blank line and get body if it is present.
                            if let Some(_) = lines.next() {
                                if let Some(b) =  lines.next() {
                                    body = Some(b.to_string());
                                }
                            }

                            debug!("received command method: {}, path: {}, body: {:?}", method, path, body);

                            // dispatch command.
                            if let Some((_id, path)) = split_id_and_path(path) {
                                self.dispatch_command(method, &path.unwrap(), body);
                            }

                            Ok(())
                        } else {
                            Err(CoreError::RequestInvalid(req.to_string()))
                        }
                    } else {
                        Err(CoreError::RequestInvalid(req.to_string()))
                    }
                } else {
                    Err(CoreError::RequestInvalid(req.to_string()))
                }
            } else {
                Err(CoreError::RequestInvalid("(no request line)".to_string()))
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
pub struct TimerEntry {
    pub sender: mpsc::Sender<NexusToProto>,
    pub protocol: ProtocolType,
    pub expiration: Instant,
    pub token: u32,
}

/// Timer entry implementation.
impl TimerEntry {

    /// Constructor.
    pub fn new(p: ProtocolType, sender: mpsc::Sender<NexusToProto>, d: Duration, token: u32) -> TimerEntry {
        TimerEntry {
            sender: sender,
            protocol: p,
            expiration: Instant::now() + d,
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
                if let Err(err) = self.sender.send(NexusToProto::TimerExpiration(self.token)) {
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
        self.expiration
    }

    /// Set expiration.
    fn set_expiration(&mut self, d: Duration) {
        self.expiration = Instant::now() + d;
    }
}

/// RouterNexus implementation.
impl RouterNexus {

    /// Constructor.
    pub fn new() -> RouterNexus {
        RouterNexus {
            config: RefCell::new(ConfigMaster::new()),
            masters: RefCell::new(HashMap::new()),
            timer_server: RefCell::new(TimerServer::new()),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    /// Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Clone global config.
        let ref mut _config = *self.config.borrow_mut();

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

    /// Clone P2N sender mpsc.
    fn clone_sender_p2n(&self) -> mpsc::Sender<ProtoToNexus> {
        if let Some(ref mut sender_p2n) = *self.sender_p2n.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2n);
        }
        panic!("failed to clone");
    }

    /// Clone P2Z sender mpsc.
    fn _clone_sender_p2z(&self) -> mpsc::Sender<ProtoToZebra> {
        if let Some(ref mut sender_p2z) = *self.sender_p2z.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2z)
        }
        panic!("failed to clone");
    }

    /// Initialize config.
    fn config_init(&self) {
        self.config.borrow_mut().register_protocol("route_ipv4", ProtocolType::Zebra);
        self.config.borrow_mut().register_protocol("route_ipv6", ProtocolType::Zebra);

        self.config.borrow_mut().register_protocol("ospf", ProtocolType::Ospf);
    }

    /// Entry point to start RouterNexus.
    pub fn start(&self, event_manager: Arc<EventManager>) -> Result<(), CoreError> {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        self.sender_p2n.borrow_mut().replace(sender_p2n);

        // Config init.
        self.config_init();

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = self.spawn_zebra(self.clone_sender_p2n());
        self.sender_p2z.borrow_mut().replace(sender_p2z);
        self.masters.borrow_mut().insert(ProtocolType::Zebra, MasterTuple { handle, sender });


        // XXX spawn OSPF
        let (handle, sender, _sender_z2p) = self.spawn_protocol(ProtocolType::Ospf,
                                                               self.clone_sender_p2n(),
                                                               self._clone_sender_p2z());
        self.masters.borrow_mut().insert(ProtocolType::Ospf, MasterTuple { handle, sender });

        // Event loop.
        'main: loop {
            // Signal is caught.
            if signal::is_sigint_caught() {
                break 'main;
            }

            // Process events.
            match event_manager.poll() {
                Err(CoreError::SystemShutdown) => break 'main,
                _ => {}
            }

            // Process ProtoToNexus messages through channels.
            while let Ok(d) = receiver.try_recv() {
                match d {
                    ProtoToNexus::TimerRegistration((p, d, token)) => {
                        debug!("Received Timer Registration {} {}", p, token);

                        if let Some(tuple) = self.masters.borrow_mut().get(&p) {
                            let entry = TimerEntry::new(p, tuple.sender.clone(), d, token);
                            self.timer_server.borrow_mut().register(d, Rc::new(entry));
                        }
                    },
                    ProtoToNexus::ProtoException(s) => {
                        debug!("Received Exception {}", s);
                    },
                }
            }

            thread::sleep(Duration::from_millis(10));

            // Process timer.
            self.timer_server.borrow_mut().run();
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
        Err(CoreError::SystemShutdown)
    }

    /// Dispatch command request from Uds stream to protocol channel.
    fn dispatch_command(&self, method: Method, path: &str, body: Option<String>) {
        match self.config.borrow().lookup(path) {
            Some(config_or_protocol) => {
                match config_or_protocol {
                    ConfigOrProtocol::Local(_config) => {
                            debug!("local config");
                    },
                    ConfigOrProtocol::Proto(p) => {
                        match self.masters.borrow_mut().get(&p) {
                            Some(tuple) => {
                                let b = match body {
                                    Some(s) => Some(Box::new(s)),
                                    None => None
                                };

                                // XXX
                                match tuple.sender.send(NexusToProto::SendConfig((method, path.to_string(), b))) {
                                    Ok(_) => {},
                                    Err(_) => error!("sender error"),
                                }
                            },
                            None => {
                                panic!("Unexpected error");
                            },
                        }
                    },
                }
            },
            None => {
                error!("No config exists")
            }
        }
    }
}
