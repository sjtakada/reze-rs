//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra Master
//

use log::{debug, error};
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::net::{Ipv4Addr, Ipv6Addr};

//use crate::core::event::*;

use crate::core::protocols::ProtocolType;
use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;
use crate::core::message::zebra::ProtoToZebra;
use crate::core::message::zebra::ZebraToProto;
use crate::core::config_master::*;

use super::link::*;
use super::address::*;
use super::kernel::*;

/// Store Zebra Client related information.
struct ClientTuple {
    /// Channel sender from Zebra to Protocol
    _sender: mpsc::Sender<ZebraToProto>,
}

/// Zebra Master.
pub struct ZebraMaster {
    /// Reference config tree.
    config: Arc<ConfigMaster>,

    /// Kernel interface.
    kernel: RefCell<Kernel>,

    /// ProtocolType to Zebra Client Tuple Map.
    clients: RefCell<HashMap<ProtocolType, ClientTuple>>,

    /// Ifindex to Link map.
    links: RefCell<HashMap<i32, Rc<Link>>>,

    ///
    _name2ifindex: HashMap<String, i32>,
}

impl ZebraMaster {
    pub fn new(config: Arc<ConfigMaster>) -> ZebraMaster {
        let callbacks = KernelCallbacks {
            add_link: &ZebraMaster::add_link,
            delete_link: &ZebraMaster::delete_link,
            add_ipv4_address: &ZebraMaster::add_ipv4_address,
            delete_ipv4_address: &ZebraMaster::delete_ipv4_address,
            add_ipv6_address: &ZebraMaster::add_ipv6_address,
            delete_ipv6_address: &ZebraMaster::delete_ipv6_address,
        };

        ZebraMaster {
            config: config,
            kernel: RefCell::new(Kernel::new(callbacks)),
            clients: RefCell::new(HashMap::new()),
            links: RefCell::new(HashMap::new()),
            _name2ifindex: HashMap::new(),
        }
    }

    pub fn add_link(&self, link: Link) {
        debug!("New Link");

        self.links.borrow_mut().insert(link.index(), Rc::new(link));

        // TODO: notify this to other protocols.
    }

    pub fn delete_link(&self, _link: Link) {
        debug!("Delete Link");

        //self.links.borrow_mut().insert(link.index(), Rc::new(link));

        // TODO: notify this to other protocols.
    }

    pub fn add_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Add IPv4 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.add_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    pub fn delete_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Delete IPv4 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.delete_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    pub fn add_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Add IPv6 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.add_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    pub fn delete_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Delete IPv6 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.delete_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    pub fn kernel_init(master: Rc<ZebraMaster>) {
        // Init Kernel driver.
        master.kernel.borrow_mut().init(master.clone());
    }

    pub fn start(&self,
                 _sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>,
                 receiver_p2z: mpsc::Receiver<ProtoToZebra>) {
        // Initialize some stuff.

        // Zebra main loop
        'main: loop {
            // Process ProtoToZebra messages through the channel.
            while let Ok(d) = receiver_p2z.try_recv() {
                match d {
                    ProtoToZebra::RegisterProto((proto, sender_z2p)) => {
                        self.clients.borrow_mut().insert(proto, ClientTuple { _sender: sender_z2p });
                        debug!("Register Protocol {}", proto);
                    },
                    ProtoToZebra::RouteAdd(_i) => {
                    },
                    ProtoToZebra::RouteLookup(_i) => {
                    },
                }
            }

            // Process NexusToProto messages through the channel.
            while let Ok(d) = receiver_n2p.try_recv() {
                match d {
                    NexusToProto::TimerExpiration(token) => {
                        debug!("Received TimerExpiration with token {}", token);

                        /*
                        match self.timer_handler_get(token) {
                        Some(handler) => {
                        handler.handle(EventType::TimerEvent);
                    },
                        None => {
                        error!("Handler doesn't exist");
                    }
                    }
                         */
                    },
                    NexusToProto::PostConfig((path, json)) => {
                        debug!("Received PostConfig with command {} {}", path, json);
                    },
                    NexusToProto::ProtoTermination => {
                        debug!("Received ProtoTermination");
                        break 'main;
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));

            // TODO: Some cleanup has to be done for inner.
            // inner.finish();
        }
        debug!("Zebra terminated");
    }
}

