//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel driver
//

use std::rc::Rc;
use std::net::{Ipv4Addr, Ipv6Addr};

use rtable::prefix::*;

use super::rib::*;
use super::master::*;
use super::link::*;
use super::address::*;

use super::linux::netlink::*;

//use log::error;

/// Kernel Callbacks.
pub struct KernelCallbacks {
    pub add_link: &'static dyn Fn(&ZebraMaster, Link) -> (),
    pub delete_link: &'static dyn Fn(&ZebraMaster, Link) -> (),
    pub add_ipv4_address: &'static dyn Fn(&ZebraMaster, i32, Connected<Ipv4Addr>) -> (),
    pub delete_ipv4_address: &'static dyn Fn(&ZebraMaster, i32, Connected<Ipv4Addr>) -> (),
    pub add_ipv6_address: &'static dyn Fn(&ZebraMaster, i32, Connected<Ipv6Addr>) -> (),
    pub delete_ipv6_address: &'static dyn Fn(&ZebraMaster, i32, Connected<Ipv6Addr>) -> (),
}

/// Kernel driver.
pub struct Kernel {
    /// Kernel driver for Linux.
    driver: Netlink,
}

impl Kernel {
    pub fn new(callbacks: KernelCallbacks) -> Kernel {
        let netlink = Netlink::new(callbacks).unwrap();

        Kernel {
            driver: netlink,
        }
    }

    pub fn init(&mut self, master: Rc<ZebraMaster>) {
        self.driver.set_master(master);

        let _links = self.driver.get_links_all();
        let _v4addr = self.driver.get_addresses_all::<Ipv4Addr>();
        let _v6addr = self.driver.get_addresses_all::<Ipv6Addr>();
        // route ipv4
        // route ipv6
    }

    pub fn install<T: AddressLen + Clone>(&self, prefix: &Prefix<T>, rib: &Rib<T>) {
        self.driver.install(prefix, rib);
    }
}
