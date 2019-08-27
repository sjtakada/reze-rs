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
//use log::error;

use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::boxed::Box;
use std::cell::RefCell;
use std::time::Duration;
//use std::time::Instant;

use super::error::*;
use super::event::*;
use super::uds_server::*;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

use super::master::ProtocolMaster;
use super::config::Config;
use super::config_global::ConfigGlobal;
use crate::zebra::master::ZebraMaster;
use crate::zebra::static_route::*;
use crate::bgp::master::BgpMaster;
use crate::ospf::master::OspfMasterInner;

use super::timer;

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
    config: RefCell<Arc<ConfigGlobal>>,

    /// MasterInner map
    masters: RefCell<HashMap<ProtocolType, MasterTuple>>,

    /// Timer server
    timer_server: RefCell<timer::Server>,

    /// Sender channel for ProtoToNexus
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    /// Sender channel for ProtoToZebra
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,
}

impl UdsServerHandler for RouterNexus {
    // Process command.
    fn handle_message(&self, _server: Arc<UdsServer>, entry: &UdsServerEntry) -> Result<(), CoreError> {
        if let Some(command) = entry.stream_read() {
            debug!("received command {}", command);

            if command == "ospf" {
                    // Spawn ospf instance
                    let (handle, sender, _sender_z2p) =
                        self.spawn_protocol(ProtocolType::Ospf,
                                            self.clone_sender_p2n(),
                                            self.clone_sender_p2z());
                    self.masters.borrow_mut().insert(ProtocolType::Ospf, MasterTuple { handle, sender });

                    // register sender_z2p to Zebra thread
            } else if command == "quie" {
                return Err(CoreError::NexusTermination)
            } else {
                return Err(CoreError::CommandNotFound(command.to_string()))
            }
        }

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
    }
*/
        Ok(())
    }

    fn handle_connect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), CoreError> {
        Ok(())
    }
    
    fn handle_disconnect(&self, _server: Arc<UdsServer>, _entry: &UdsServerEntry) -> Result<(), CoreError> {
        Ok(())
    }
}

impl RouterNexus {
    /// Constructor.
    pub fn new() -> RouterNexus {
        let config = RouterNexus::config_global();

        RouterNexus {
            config: RefCell::new(Arc::new(config)),
            masters: RefCell::new(HashMap::new()),
            timer_server: RefCell::new(timer::Server::new()),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    // Initialize Config tree.
    fn config_global() -> ConfigGlobal {
        let mut config = ConfigGlobal::new();
        let ipv4_routes = Ipv4StaticRoute::new();
        config.register_child(Arc::new(ipv4_routes));

        config
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Clone global config.
        let ref mut config = *self.config.borrow_mut();
        let config = Arc::clone(config);

        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let zebra = Rc::new(ZebraMaster::new(config));
            ZebraMaster::kernel_init(zebra.clone());
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
    fn finish_protocol(&self, proto: &ProtocolType) {
        if let Some(tuple) = self.masters.borrow_mut().remove(&proto) {
            match tuple.sender.send(NexusToProto::ProtoTermination) {
                Ok(_) => {},
                Err(_) => {},
            }

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

    //
    pub fn start(&self, event_manager: Arc<EventManager>) {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        self.sender_p2n.borrow_mut().replace(sender_p2n);

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = self.spawn_zebra(self.clone_sender_p2n());
        self.sender_p2z.borrow_mut().replace(sender_p2z);
        self.masters.borrow_mut().insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        'main: loop {
            //
            match event_manager.poll() {
                Err(CoreError::NexusTermination) => break 'main,
                _ => {}
            }

            // Process channels
            while let Ok(d) = receiver.try_recv() {
                match d {
                    ProtoToNexus::TimerRegistration((p, d, token)) => {
                        debug!("Received timer registration {} {}", p, token);

                        self.timer_server.borrow_mut().register(p, d, token);
                    }
                    ProtoToNexus::ProtoException(s) => {
                        debug!("Received exception {}", s);
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));

            // Process timer
            match self.timer_server.borrow_mut().pop_if_expired() {
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
