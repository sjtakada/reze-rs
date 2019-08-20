//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel interface
//

use std::rc::Weak;
use std::rc::Rc;
use std::cell::Cell;

use super::master::*;
use super::link::*;
use super::address::*;

use super::linux::netlink::*;

/// Kernel interface.
pub struct Kernel {
    // Zebra Master.
    master: Weak<ZebraMaster>,

    /// Netlink socket.
    netlink: Netlink,
}

impl Kernel {
    pub fn new() -> Kernel {
        let netlink = Netlink::new().unwrap();

        Kernel {
            master: Default::default(),
            netlink,
        }
    }

    pub fn connect(&mut self, master: Rc<ZebraMaster>) {
        self.master = Rc::downgrade(&master);
    }

    pub fn init(&self) {
//        self.master = Some(master);

        let master = self.master.upgrade();
        match master {
            Some(master) => println!("Some"),
            Nonw => println!("None"),
        }

        let links = self.netlink.get_links_all().unwrap();
        let v4addr = self.netlink.get_ipv4_addresses_all();
        let v6addr = self.netlink.get_ipv6_addresses_all();
    }
}
