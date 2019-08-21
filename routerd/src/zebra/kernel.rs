//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel driver
//

use std::rc::Weak;
use std::rc::Rc;

use super::master::*;
use super::link::*;
use super::address::*;

use super::linux::netlink::*;

//use log::error;

/// Kernel Callbacks.
pub struct KernelCallbacks {
    pub new_link: &'static Fn(&ZebraMaster, Link) -> (),
    pub delete_link: &'static Fn(&ZebraMaster, Link) -> (),
}

/// Kernel interface.
pub struct Kernel {
    /// Netlink for Linux.
    netlink: Netlink,
}

impl Kernel {
    pub fn new(callbacks: KernelCallbacks) -> Kernel {
        let netlink = Netlink::new(callbacks).unwrap();

        Kernel {
            netlink,
        }
    }

    pub fn init(&mut self, master: Rc<ZebraMaster>) {
        self.netlink.set_master(master);

        let links = self.netlink.get_links_all();
        let v4addr = self.netlink.get_ipv4_addresses_all();
        let v6addr = self.netlink.get_ipv6_addresses_all();
    }
}
