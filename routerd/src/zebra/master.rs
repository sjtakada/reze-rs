//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra Master
//

use log::{debug, error};
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;
use std::hash::Hash;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc;
use std::net::{Ipv4Addr, Ipv6Addr};

//use crate::core::event::*;

use rtable::prefix::*;

use crate::core::protocols::ProtocolType;
use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;
use crate::core::message::zebra::ProtoToZebra;
use crate::core::message::zebra::ZebraToProto;
use crate::core::config_master::*;

use super::link::*;
use super::address::*;
use super::kernel::*;
use super::static_route::*;
use super::rib::*;
//use super::nexthop::*;

/// Store Zebra Client related information.
struct ClientTuple {
    /// Channel sender from Zebra to Protocol
    _sender: mpsc::Sender<ZebraToProto>,
}

/// Zebra Master.
pub struct ZebraMaster {
    /// Reference config tree.
    config: RefCell<ConfigMaster>,

    /// Kernel interface.
    kernel: RefCell<Kernel>,

    /// ProtocolType to Zebra Client Tuple Map.
    clients: RefCell<HashMap<ProtocolType, ClientTuple>>,

    /// Ifindex to Link map.
    links: RefCell<HashMap<i32, Rc<Link>>>,

    /// TBD
    _name2ifindex: HashMap<String, i32>,

    /// IPv4 RIB.
    rib_ipv4: RefCell<RibTable<Ipv4Addr>>,

    /// IPv6 RIB.
    rib_ipv6: RefCell<RibTable<Ipv6Addr>>,
}

impl ZebraMaster {
    pub fn new() -> ZebraMaster {
        let callbacks = KernelCallbacks {
            add_link: &ZebraMaster::add_link,
            delete_link: &ZebraMaster::delete_link,
            add_ipv4_address: &ZebraMaster::add_ipv4_address,
            delete_ipv4_address: &ZebraMaster::delete_ipv4_address,
            add_ipv6_address: &ZebraMaster::add_ipv6_address,
            delete_ipv6_address: &ZebraMaster::delete_ipv6_address,
        };

        ZebraMaster {
            config: RefCell::new(ConfigMaster::new()),
            kernel: RefCell::new(Kernel::new(callbacks)),
            clients: RefCell::new(HashMap::new()),
            links: RefCell::new(HashMap::new()),
            _name2ifindex: HashMap::new(),
            rib_ipv4: RefCell::new(RibTable::<Ipv4Addr>::new()),
            rib_ipv6: RefCell::new(RibTable::<Ipv6Addr>::new()),
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

    pub fn rib_add_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB add static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();
        let mut map = Rib::<Ipv4Addr>::from_static_route(sr);

        for (_, rib) in map.drain() {

            // TBD: handle return value
            self.rib_ipv4.borrow_mut().add(&prefix, rib);
        }
    }

    pub fn rib_delete_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB delete static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();

        // TBD: handle return value
        self.rib_ipv4.borrow_mut().delete(&prefix);
    }

    pub fn rib_install_kernel<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable + Clone + FromStr + Hash + Eq
    {
        self.kernel.borrow_mut().install(prefix, rib);
    }

    pub fn rib_update_kernel<T>(&self, prefix: &Prefix<T>, new: &Rib<T>, old: &Rib<T>)
    where T: Addressable + Clone + FromStr + Hash + Eq
    {
        self.kernel.borrow_mut().uninstall(prefix, old);
        self.kernel.borrow_mut().install(prefix, old);
    }

    pub fn rib_uninstall_kernel<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable + Clone + FromStr + Hash + Eq
    {
        self.kernel.borrow_mut().uninstall(prefix, rib);
    }

    pub fn init(master: Rc<ZebraMaster>) {
        ZebraMaster::kernel_init(master.clone());
        ZebraMaster::config_init(master.clone());
        ZebraMaster::rib_init(master.clone());
    }

    fn kernel_init(master: Rc<ZebraMaster>) {
        // Init Kernel driver.
        master.kernel.borrow_mut().init(master.clone());
    }

    fn config_init(master: Rc<ZebraMaster>) {
        let ipv4_routes = Ipv4StaticRoute::new(master.clone());
        master.config.borrow_mut().register_config("route_ipv4", Rc::new(ipv4_routes));
    }

    fn rib_init(master: Rc<ZebraMaster>) {
        master.rib_ipv4.borrow_mut().set_master(master.clone());
        master.rib_ipv6.borrow_mut().set_master(master.clone());
    }

    pub fn start(&self,
                 _sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>,
                 receiver_p2z: mpsc::Receiver<ProtoToZebra>) {
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
                    NexusToProto::SendConfig((method, path, body)) => {
                        debug!("Received SendConfig with command {} {} {:?}", method, path, body);

                        match self.config.borrow_mut().apply(method, &path, body) {
                            Ok(_) => { }
                            Err(err) => error!("{}", err.to_string()),
                        }
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

