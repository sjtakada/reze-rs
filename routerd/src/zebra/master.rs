//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra Master
//

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::mpsc;
use std::net::{Ipv4Addr, Ipv6Addr};

use log::{debug, error};
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
use super::error::ZebraError;

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

    /// Link Master.
    link_master: RefCell<LinkMaster>,

    /// IPv4 RIB.
    rib_ipv4: RefCell<RibTable<Ipv4Addr>>,

    /// IPv6 RIB.
    rib_ipv6: RefCell<RibTable<Ipv6Addr>>,
}

impl ZebraMaster {

    /// Constructor.
    pub fn new() -> ZebraMaster {
        ZebraMaster {
            mds: RefCell::new(Rc::new(MdsNode::new("ZebraMaster"))),
            kernel: RefCell::new(Kernel::new()),
            clients: RefCell::new(HashMap::new()),
            link_master: RefCell::new(LinkMaster::new()),
            rib_ipv4: RefCell::new(RibTable::<Ipv4Addr>::new()),
            rib_ipv6: RefCell::new(RibTable::<Ipv6Addr>::new()),
        }
    }

    pub fn rib_ipv4(&self) -> RefMut<RibTable<Ipv4Addr>> {
        self.rib_ipv4.borrow_mut()
    }

    pub fn rib_ipv6(&self) -> RefMut<RibTable<Ipv6Addr>> {
        self.rib_ipv6.borrow_mut()
    }

    /// Get Add link from kernel.
    pub fn get_add_link(&self, kl: KernelLink) {
        debug!("New Link");

        self.link_master.borrow_mut().add_link(Link::from_kernel(kl));

        // TODO: notify this to other protocols.
    }

    /// Get Delete link from kernel.
    pub fn get_delete_link(&self, kl: KernelLink) {
        debug!("Delete Link");

        self.link_master.borrow_mut().delete_link(Link::from_kernel(kl));

        // TODO: notify this to other protocols.
    }

    /// Get Add IPv4 Adress from kernel.
    pub fn get_add_ipv4_address(&self, ka: KernelAddr<Ipv4Addr>) {
        debug!("Add Ipv4Address");

        let index = ka.ifindex;

        self.link_master.borrow_mut().add_ipv4_address(index, Connected::<Ipv4Addr>::from_kernel(ka));
    }

    /// Get Delete IPv4 Adress from kernel.
    pub fn get_delete_ipv4_address(&self, ka: KernelAddr<Ipv4Addr>) {
        debug!("Add Ipv4Address");

        let index = ka.ifindex;

        self.link_master.borrow_mut().delete_ipv4_address(index, Connected::<Ipv4Addr>::from_kernel(ka));
    }

    /// Get Add IPv6 Adress from kernel.
    pub fn get_add_ipv6_address(&self, ka: KernelAddr<Ipv6Addr>) {
        debug!("Add Ipv6Address");

        let index = ka.ifindex;

        self.link_master.borrow_mut().add_ipv6_address(index, Connected::<Ipv6Addr>::from_kernel(ka));
    }

    /// Get Delete IPv6 Adress from kernel.
    pub fn get_delete_ipv6_address(&self, ka: KernelAddr<Ipv6Addr>) {
        debug!("Add Ipv6Address");

        let index = ka.ifindex;

        self.link_master.borrow_mut().delete_ipv6_address(index, Connected::<Ipv6Addr>::from_kernel(ka));
    }

/*
    /// Add link.
    pub fn add_link(&self, link: Link) {
        self.link_master.borrow_mut().add_link(link);

        // TODO: notify this to other protocols.
    }

    /// Delete link.
    pub fn delete_link(&self, link: Link) {
        debug!("Delete Link");

        self.link_master.borrow_mut().delete_link(link);

        // TODO: notify this to other protocols.
    }

    /// Add IPv4 address.
    pub fn add_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Add IPv4 Address");

        self.link_master.borrow_mut().add_ipv4_address(index, conn);
    }

    /// Delete IPv4 address.
    pub fn delete_ipv4_address(&self, index: i32, conn: Connected<Ipv4Addr>) {
        debug!("Delete IPv4 Address");

        self.link_master.borrow_mut().delete_ipv4_address(index, conn);
    }

    /// Add IPv6 address.
    pub fn add_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Add IPv6 Address");

        self.link_master.borrow_mut().add_ipv6_address(index, conn);
    }

    /// Delete IPv6 address.
    pub fn delete_ipv6_address(&self, index: i32, conn: Connected<Ipv6Addr>) {
        debug!("Delete IPv6 Address");

        self.link_master.borrow_mut().delete_ipv6_address(index, conn);
    }
*/


    /// Add RIB for IPv4 static route.
    pub fn rib_add_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB add static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();
        let mut map = Rib::<Ipv4Addr>::from_static_route(sr);

        let mut rib_ipv4 = self.rib_ipv4.borrow_mut();

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

    /// Delete RIB for IPv4 static route.
    pub fn rib_delete_static_ipv4(&self, sr: Arc<StaticRoute<Ipv4Addr>>) {
        debug!("RIB delete static IPv4 {:?}", sr.prefix());

        let prefix = sr.prefix().clone();
        let mut map = Rib::<Ipv4Addr>::from_static_route(sr);

        let mut rib_ipv4 = self.rib_ipv4.borrow_mut();

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
        // Register callbacks.

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_add_link(
            Box::new(move |kl: KernelLink| -> Result<(), ZebraError> {
                // TBD error hanadling
                clone.get_add_link(kl);
                Ok(())
            }));

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_delete_link(
            Box::new(move |kl: KernelLink| -> Result<(), ZebraError> {
                // TBD error handling.
                clone.get_delete_link(kl);
                Ok(())
            }));

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_add_ipv4_address(
            Box::new(move |ka: KernelAddr<Ipv4Addr>| -> Result<(), ZebraError> {
                // TBD error handling.
                clone.get_add_ipv4_address(ka);
                Ok(())
            }));

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_delete_ipv4_address(
            Box::new(move |ka: KernelAddr<Ipv4Addr>| -> Result<(), ZebraError> {
                // TBD error handling.
                clone.get_delete_ipv4_address(ka);
                Ok(())
            }));

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_add_ipv6_address(
            Box::new(move |ka: KernelAddr<Ipv6Addr>| -> Result<(), ZebraError> {
                // TBD error handling.
                clone.get_add_ipv6_address(ka);
                Ok(())
            }));

        let clone = master.clone();
        master.kernel.borrow_mut().driver().register_delete_ipv6_address(
            Box::new(move |ka: KernelAddr<Ipv6Addr>| -> Result<(), ZebraError> {
                // TBD error handling.
                clone.get_delete_ipv6_address(ka);
                Ok(())
            }));

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
        let rib_table_ipv4 = Rc::new(RibTableIpv4::new(master.clone()));

        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv4", rib_table_ipv4.clone());
//        MdsNode::register_handler(mds.clone(), "/exec/show/route_ipv4", rib_table_ipv4.clone());
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
                            Err(err) => Some(Box::new(err.json_status()))
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
                            Err(err) => Some(Box::new(err.json_status()))
                        };

                        if let Err(_err) = sender_p2n.send(ProtoToNexus::ExecResponse((index, resp))) {
                            error!("Sender error: ProtoToNexus::ExecResponse");
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

