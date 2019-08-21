//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - IPv4 and IPv6 address handler.
//

use std::io;
//use std::net::{Ipv4Addr, Ipv6Addr};
use rtable::prefix::*;

/// Trait IP address handler.
pub trait AddressHandler {
    /// Get all IPv4 addresses from kernel.
    fn get_ipv4_addresses_all(&self) -> Result<(), io::Error>;

    /// Get all IPv6 addresses from kernel.
    fn get_ipv6_addresses_all(&self) -> Result<(), io::Error>;


}

/// Connected Address.
pub struct Connected<T> {
    /// Address prefix.
    address: Prefix<T>,

    /// Destination address prefix for peer.
    destination: Option<Prefix<T>>,

    /// Secondary address.
    secondary: bool,

    /// Unnumbered.
    unnumbered: bool,

    /// Label.
    label: Option<String>,
}

impl<T> Connected<T> {
    pub fn new(prefix: Prefix<T>) -> Connected<T> {
        Connected::<T> {
            address: prefix,
            destination: None,
            secondary: false,
            unnumbered: false,
            label: None,
        }
    }
}
