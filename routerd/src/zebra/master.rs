//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra Master
//

use log::{debug, error};
use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc;
use std::net::{Ipv4Addr, Ipv6Addr};

use rtable::prefix::*;

use crate::core::protocols::ProtocolType;
use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;
use crate::core::message::zebra::ProtoToZebra;
use crate::core::message::zebra::ZebraToProto;
use crate::core::mds::*;

use super::link::*;
use super::address::*;
use super::kernel::*;
use super::static_route::*;
use super::rib::*;

/// Store Zebra Client related information.
struct ClientTuple {

    /// Channel sender from Zebra to Protocol
    _sender: mpsc::Sender<ZebraToProto>,
}

/// Zebra Master.
pub struct ZebraMaster {

    /// Mds Config and Exec.
    mds: RefCell<Rc<MdsNode>>,

    /// Kernel interface.
    kernel: RefCell<Kernel>,

    /// ProtocolType to Zebra Client Tuple Map.
    clients: RefCell<HashMap<ProtocolType, ClientTuple>>,

    /// Ifindex to Link map.
    links: RefCell<HashMap<i32, Rc<Link>>>,

    /// TBD
    _name2ifindex: HashMap<String, i32>,

    /// IPv4 RIB.
    rib_ipv4: RefCell<Rc<RibTable<Ipv4Addr>>>,

    /// IPv6 RIB.
    _rib_ipv6: RefCell<Rc<RibTable<Ipv6Addr>>>,
}

/// Zebra Master implementation.
impl ZebraMaster {

    /// Constructor.
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
            mds: RefCell::new(Rc::new(MdsNode::new())),
            kernel: RefCell::new(Kernel::new(callbacks)),
            clients: RefCell::new(HashMap::new()),
            links: RefCell::new(HashMap::new()),
            _name2ifindex: HashMap::new(),
            rib_ipv4: RefCell::new(Rc::new(RibTable::<Ipv4Addr>::new())),
            _rib_ipv6: RefCell::new(Rc::new(RibTable::<Ipv6Addr>::new())),
        }
    }

    /// Add link.
    pub fn add_link(&self, link: Link) {
        debug!("New Link");

        self.links.borrow_mut().insert(link.index(), Rc::new(link));

        // TODO: notify this to other protocols.
    }

    /// Delete link.
    pub fn delete_link(&self, _link: Link) {
        debug!("Delete Link");

        //self.links.borrow_mut().insert(link.index(), Rc::new(link));

        // TODO: notify this to other protocols.
    }

    /// Add IPv4 address.
    pub fn add_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Add IPv4 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.add_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Delete IPv4 address.
    pub fn delete_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Delete IPv4 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.delete_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Add IPv6 address.
    pub fn add_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Add IPv6 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.add_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Delete IPv6 address.
    pub fn delete_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Delete IPv6 Address");

        match self.links.borrow().get(&index) {
            Some(link) => link.delete_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Add RIB for IPv4 static route.
    pub fn rib_add_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB add static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();
        let mut map = Rib::<Ipv4Addr>::from_static_route(sr);

        let mut rib_ipv4 = self.rib_ipv4.borrow_mut();
        if let Some(rib_ipv4) = Rc::get_mut(&mut rib_ipv4) {
            for (_, rib) in map.drain() {
                rib_ipv4.add(&prefix, rib);
            }

            rib_ipv4.process(&prefix, |prefix: &Prefix<Ipv4Addr>, entry: &RibEntry<Ipv4Addr>| {
                if let Some(ref mut fib) = *entry.fib() {
                    self.rib_uninstall_kernel(prefix, &fib);
                }

                if let Some(selected) = entry.select() {
                    self.rib_install_kernel(prefix, &selected);
                    Some(selected)
                } else {
                    None
                }
            });
        }
    }

    /// Delete RIB for IPv4 static route.
    pub fn rib_delete_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB delete static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();
        let mut map = Rib::<Ipv4Addr>::from_static_route(sr);

        let mut rib_ipv4 = self.rib_ipv4.borrow_mut();
        if let Some(rib_ipv4) = Rc::get_mut(&mut rib_ipv4) {
            for (_, rib) in map.drain() {
                rib_ipv4.delete(&prefix, rib);
            }

            rib_ipv4.process(&prefix, |prefix: &Prefix<Ipv4Addr>, entry: &RibEntry<Ipv4Addr>| {
                if let Some(ref mut fib) = *entry.fib() {
                    self.rib_uninstall_kernel(prefix, &fib);
                }

                if let Some(selected) = entry.select() {
                    self.rib_install_kernel(prefix, &selected);
                    Some(selected)
                } else {
                    None
                }
            });
        }
    }

    /// Install a route for given RIB to kernel.
    pub fn rib_install_kernel<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable
    {
        self.kernel.borrow_mut().install(prefix, rib);
    }

    /// Update a route for given RIB to kkernel.
    pub fn rib_update_kernel<T>(&self, prefix: &Prefix<T>, new: &Rib<T>, old: &Rib<T>)
    where T: Addressable
    {
        self.kernel.borrow_mut().uninstall(prefix, old);
        self.kernel.borrow_mut().install(prefix, new);
    }

    /// Uninstall a route for given RIB from kernel.
    pub fn rib_uninstall_kernel<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable
    {
        self.kernel.borrow_mut().uninstall(prefix, rib);
    }

    /// Initialization.
    pub fn init(master: Rc<ZebraMaster>) {
        ZebraMaster::kernel_init(master.clone());
        ZebraMaster::config_init(master.clone());
        ZebraMaster::exec_init(master.clone());
        ZebraMaster::rib_init(master.clone());
    }

    /// Kernel layer initialization.
    fn kernel_init(master: Rc<ZebraMaster>) {
        // Init Kernel driver.
        master.kernel.borrow_mut().init(master.clone());
    }

    /// Initiialize configuration.
    fn config_init(master: Rc<ZebraMaster>) {
        let mds = master.mds.borrow().clone();
        let ipv4_routes = Rc::new(Ipv4StaticRoute::new(master.clone()));

        MdsNode::register_handler(mds.clone(), "/config/route_ipv4", ipv4_routes.clone());
    }

    /// Initialize exec.
    fn exec_init(master: Rc<ZebraMaster>) {
        let mds = master.mds.borrow().clone();
        let rib_ipv4 = master.rib_ipv4.borrow();

        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv4", rib_ipv4.clone());
    }

    /// Initialize RIB.
    fn rib_init(_master: Rc<ZebraMaster>) {

    }

    /// Entry point of zebra master.
    pub fn start(&self,
                 sender_p2n: mpsc::Sender<ProtoToNexus>,
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
                    NexusToProto::ConfigRequest((index, method, path, body)) => {
                        debug!("Received ConfigRequest with command {} {} {} {:?}", index, method, path, body);

                        let mds = self.mds.borrow().clone();
                        let resp = match MdsNode::handle(mds, index, method, &path, body) {
                            Ok(s) => {
                                match s {
                                    Some(s) => Some(Box::new(s)),
                                    None => None,
                                }
                            },
                            Err(err) => Some(Box::new(err.to_string())),
                        };

                        if let Err(_err) = sender_p2n.send(ProtoToNexus::ConfigResponse((index, resp))) {
                            error!("Sender error: ProtoToNexus::ConfigResponse");
                        }
                    },
                    NexusToProto::ExecRequest((index, method, path, body)) => {
                        debug!("Received ExecRequest with command {} {} {} {:?}", index, method, path, body);

                        let mds = self.mds.borrow().clone();
                        let resp = match MdsNode::handle(mds, index, method, &path, body) {
                            Ok(s) => {
                                match s {
                                    Some(s) => Some(Box::new(s)),
                                    None => None,
                                }
                            },
                            Err(err) => Some(Box::new(err.to_string())),
                        };

                        if let Err(_err) = sender_p2n.send(ProtoToNexus::ExecResponse((index, resp))) {
                            error!("Sender error: ProtoToNexus::ConfigResponse");
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

