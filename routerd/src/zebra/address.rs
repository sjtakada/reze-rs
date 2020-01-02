//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - IPv4 and IPv6 address handler.
//

use std::str::FromStr;
use std::net::{Ipv4Addr, Ipv6Addr};

use rtable::prefix::*;

use super::error::*;

pub trait AddressFamily {
    /// Return libc Addressfamily
    fn address_family() -> libc::c_int;
}

impl AddressFamily for Ipv4Addr {
    fn address_family() -> libc::c_int {
        libc::AF_INET
    }
}

impl AddressFamily for Ipv6Addr {
    fn address_family() -> libc::c_int {
        libc::AF_INET6
    }
}

/// Trait IP address handler.
pub trait AddressHandler {
    fn get_addresses_all<T: AddressFamily + Addressable + FromStr>(&self) ->  Result<(), ZebraError>;
}

/// Connected Address.
pub struct Connected<T> {
    /// Address prefix.
    address: Prefix<T>,

    /// Destination address prefix for peer.
    _destination: Option<Prefix<T>>,

    /// Secondary address.
    _secondary: bool,

    /// Unnumbered.
    _unnumbered: bool,

    /// Label.
    _label: Option<String>,
}

impl<T> Connected<T> {
    pub fn new(prefix: Prefix<T>) -> Connected<T> {
        Connected::<T> {
            address: prefix,
            _destination: None,
            _secondary: false,
            _unnumbered: false,
            _label: None,
        }
    }

    pub fn address(&self) -> &Prefix<T> {
        &self.address
    }
}
