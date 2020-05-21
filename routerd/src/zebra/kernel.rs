//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - Kernel abstraction layer.
//

use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use log::error;
use quick_error::*;
use rtable::prefix::*;

use common::nexthop::*;

use super::rib::*;

#[cfg(target_os = "linux")]
use super::linux::netlink::*;

// Kernel Error
quick_error! {
    #[derive(Debug)]
    pub enum KernelError {
        Other(s: String) {
            description("Other error")
            display(r#"Other error {}"#, s)
        }
        System(s: String) {
            description("System error")
            display(r#"System error {}"#, s)
        }
        Route(s: String) {
            description("Route error")
            display(r#"Route error {}"#, s)
        }
        Link(s: String) {
            description("Link error")
            display(r#"Link error {}"#, s)
        }
        Address(s: String) {
            description("Address error")
            display(r#"Address error {}"#, s)
        }
        Encode(s: String) {
            description("Encode error")
            display(r#"Encode error {}"#, s)
        }
    }
}

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

/// Kernel Route Abstraction.
pub struct KernelRoute<T: Addressable> {

    /// Destination network prefix.
    pub destination: Prefix<T>,

    /// Outgoing interface index.
    pub ifindex: Option<i32>,

    /// Gateway Address.
    pub gateway: Option<T>,

    /// Metric.
    pub metric: Option<u32>,

    /// Table ID.
    pub table_id: Option<i32>,

    /// Nexthops.
    pub nexthops: Vec<Nexthop<T>>,

    /// Self route flag
    pub is_self: bool,
}

impl<T: Addressable> KernelRoute<T> {

    /// Constructor.
    pub fn new(destination: Prefix<T>) -> KernelRoute<T> {
        KernelRoute {
            destination: destination,
            ifindex: None,
            gateway: None,
            metric: None,
            table_id: None,
            nexthops: Vec::new(),
            is_self: false,
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

    /// Register Add IPv4 route callback function.
    fn register_add_ipv4_route(&self, f: Box<dyn Fn(KernelRoute<Ipv4Addr>)>);

    /// Register Delete IPv4 route callback function.
    fn register_delete_ipv4_route(&self, f: Box<dyn Fn(KernelRoute<Ipv4Addr>)>);

    /// Register Add IPv6 route callback function.
    fn register_add_ipv6_route(&self, f: Box<dyn Fn(KernelRoute<Ipv6Addr>)>);

    /// Register Delete IPv6 route callback function.
    fn register_delete_ipv6_route(&self, f: Box<dyn Fn(KernelRoute<Ipv6Addr>)>);


    /// Send a command to kernel to retrieve all link information.
    fn get_link_all(&self) -> Result<(), KernelError>;

    /// Set MTU.
    fn set_mtu(&self, mtu: u16) -> bool; // ? Error

    /// Set link up.
    fn set_link_up(&self) -> bool;

    /// Set link down.
    fn set_link_down(&self) -> bool;


    /// Get all IPv4 addresses from system.
    fn get_ipv4_address_all(&self) -> Result<(), KernelError>;

    /// Get all IPv6 addresses from system.
    fn get_ipv6_address_all(&self) -> Result<(), KernelError>;


    /// Add an IPv4 route to system.
    fn add_ipv4_route(&self, prefix: &Prefix<Ipv4Addr>, rib: &Rib<Ipv4Addr>);

    /// Delete an IPv4 route from system.
    fn delete_ipv4_route(&self, prefix: &Prefix<Ipv4Addr>, rib: &Rib<Ipv4Addr>);

    /// Add an IPv6 route to system.
    fn add_ipv6_route(&self, prefix: &Prefix<Ipv6Addr>, rib: &Rib<Ipv6Addr>);

    /// Delete an IPv6 route from system.
    fn delete_ipv6_route(&self, prefix: &Prefix<Ipv6Addr>, rib: &Rib<Ipv6Addr>);
}

/// Kernel driver.
pub struct Kernel {

    /// Kernel driver for Linux.
    driver: Arc<dyn KernelDriver>,
}

/// Kernel implementation.
impl Kernel {

    /// Constructor.
    pub fn new() -> Kernel {
        if let Some(driver) = get_driver() {
            Kernel {
                driver: Arc::new(driver),
            }
        } else {
            panic!("Failed to intitialize Kernel");
        }
    }

    /// Initialization.
    pub fn init(&mut self) {

        if let Err(err) = self.driver.get_link_all() {
            error!("Kernel get_link_all error {}", err);
        }

        if let Err(err) = self.driver.get_ipv4_address_all() {
            error!("Kernel get_get_ipv4_address_all error {}", err);
        }

        if let Err(err) = self.driver.get_ipv6_address_all() {
            error!("Kernel get_get_ipv6_address_all error {}", err);
        }

        // route ipv4
        // route ipv6
    }

    /// Return driver.
    pub fn driver(&self) -> Arc<dyn KernelDriver> {
        self.driver.clone()
    }

    /// Install an IPv4 route through driver.
    pub fn ipv4_route_install(&self, prefix: &Prefix<Ipv4Addr>, new: &Rib<Ipv4Addr>) {
        self.driver.add_ipv4_route(prefix, new);
    }

    /// Update an IPv4 route through driver.
    pub fn ipv4_route_update(&self, prefix: &Prefix<Ipv4Addr>, new: &Rib<Ipv4Addr>, old: &Rib<Ipv4Addr>) {
        self.driver.delete_ipv4_route(prefix, old);
        self.driver.add_ipv4_route(prefix, new);
    }

    /// Uninstall an IPv4 route through driver.
    pub fn ipv4_route_uninstall(&self, prefix: &Prefix<Ipv4Addr>, old: &Rib<Ipv4Addr>) {
        self.driver.delete_ipv4_route(prefix, old);
    }

    /// Install an IPv6 route through driver.
    pub fn ipv6_route_install(&self, prefix: &Prefix<Ipv6Addr>, new: &Rib<Ipv6Addr>) {
        self.driver.add_ipv6_route(prefix, new);
    }

    /// Update an IPv6 route through driver.
    pub fn ipv6_route_update(&self, prefix: &Prefix<Ipv6Addr>, new: &Rib<Ipv6Addr>, old: &Rib<Ipv6Addr>) {
        self.driver.delete_ipv6_route(prefix, old);
        self.driver.add_ipv6_route(prefix, new);
    }

    /// Uninstall an IPv6 route through driver.
    pub fn ipv6_route_uninstall(&self, prefix: &Prefix<Ipv6Addr>, old: &Rib<Ipv6Addr>) {
        self.driver.delete_ipv6_route(prefix, old);
    }
}
