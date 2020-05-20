//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - Kernel abstraction layer.
//

use std::net::{Ipv4Addr, Ipv6Addr};

use rtable::prefix::*;

use super::rib::*;
use super::link::*;
use super::address::*;
use super::linux::netlink::*;

/// Kernel Link Abstraction.
pub struct KernelLink {

    /// Inerface Index.
    pub ifindex: i32,

    /// Interface name.
    pub name: String,

    /// Interface Type.
    pub hwtype: u16,

    /// Hardward Address.
    pub hwaddr: [u8; 6],

    /// MTU.
    pub mtu: u32,
}

impl KernelLink {

    /// Constructor.
    pub fn new(index: i32, name: &str, hwtype: u16, hwaddr: [u8; 6], mtu: u32) -> KernelLink {
        KernelLink {
            ifindex: index,
            name: String::from(name),
            hwtype: hwtype,
            hwaddr: hwaddr,
            mtu: mtu,
        }
    }
}

/// Kernel Address Abstraction.
pub struct KernelAddr<T: Addressable> {

    /// Interface Index.
    pub ifindex: i32,

    /// Address prefix.
    pub address: Prefix<T>,

    /// Destination address prefix for peer.
    pub destination: Option<Prefix<T>>,

    /// Secondary address.
    pub secondary: bool,

    /// Unnumbered.
    pub unnumbered: bool,

    /// Label.
    pub label: Option<String>,
}

impl<T: Addressable> KernelAddr<T> {

    /// Constructor.
    pub fn new(ifindex: i32, prefix: Prefix<T>, destination: Option<Prefix<T>>,
               secondary: bool, unnumbered: bool, label: Option<String>) -> KernelAddr<T> {
        KernelAddr::<T> {
            ifindex: ifindex,
            address: prefix,
            destination: destination,
            secondary: secondary,
            unnumbered: unnumbered,
            label: label,
        }
    }
}

/// Kernel Driver trait.
pub trait KernelDriver {

    /// Register Add Link callback function.
    fn register_add_link(&self, f: Box<dyn Fn(KernelLink)>);

    /// Register Delete Link callback function.
    fn register_delete_link(&self, f: Box<dyn Fn(KernelLink)>);

    /// Register Add IPv4 Address callback function.
    fn register_add_ipv4_address(&self, f: Box<dyn Fn(KernelAddr<Ipv4Addr>)>);

    /// Register Delete IPv4 Address callback function.
    fn register_delete_ipv4_address(&self, f: Box<dyn Fn(KernelAddr<Ipv4Addr>)>);

    /// Register Add IPv6 Address callback function.
    fn register_add_ipv6_address(&self, f: Box<dyn Fn(KernelAddr<Ipv6Addr>)>);

    /// Register Delete IPv6 Address callback function.
    fn register_delete_ipv6_address(&self, f: Box<dyn Fn(KernelAddr<Ipv6Addr>)>);
}

/// Kernel driver.
pub struct Kernel {
    /// Kernel driver for Linux.
    driver: Netlink,
}

/// Kernel implementation.
impl Kernel {

    /// Constructor.
    pub fn new() -> Kernel {
        let netlink = Netlink::new().unwrap();

        Kernel {
            driver: netlink,
        }
    }

    /// Initialization.
    pub fn init(&mut self) {
        let _links = self.driver.get_links_all();
        let _v4addr = self.driver.get_addresses_all::<Ipv4Addr>();
        let _v6addr = self.driver.get_addresses_all::<Ipv6Addr>();
        // route ipv4
        // route ipv6
    }

    /// Return driver.
    pub fn driver(&self) -> &Netlink {
        &self.driver
    }

    /// Install route to kernel.
    pub fn install<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable
    {
        self.driver.install(prefix, rib);
    }

    /// Uninstall route from kernel.
    pub fn uninstall<T>(&self, prefix: &Prefix<T>, rib: &Rib<T>)
    where T: Addressable
    {
        self.driver.uninstall(prefix, rib);
    }
}
