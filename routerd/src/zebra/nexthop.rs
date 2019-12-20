//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Nexthop.
//

use std::fmt;
use std::str::FromStr;
use rtable::prefix::*;

/// Nexthop.
pub enum Nexthop<T: AddressLen> {
    /// IP Address.
    Address(T),

    /// Interface Name.
    Ifname(String),

    /// Network Prefix - TBD: floating nexthop.
    Network(Prefix<T>),
}

impl<T: Clone + AddressLen + FromStr> Nexthop<T> {
    /// Construct Nexthop from IP address.
    pub fn from_address(address: &T) -> Nexthop<T> {
        Nexthop::<T>::Address(address.clone())
    }

    /// Construct Nexthop from IP address string.
    pub fn from_address_str(s: &str) -> Option<Nexthop<T>> {
        match T::from_str(s) {
            Ok(address) => Some(Nexthop::<T>::Address(address.clone())),
            Err(_) => None,
        }
    }

    /// Construct Nexthop from Interface name.
    pub fn from_ifname(ifname: &str) -> Nexthop<T> {
        Nexthop::<T>::Ifname(String::from(ifname))
    }
}

impl<T: AddressLen + fmt::Debug> fmt::Display for Nexthop<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Nexthop::<T>::Address(address) => {
                write!(f, "{:?}", address)
            },
            Nexthop::<T>::Ifname(ifname) => {
                write!(f, "{}", ifname)
            },
            Nexthop::<T>::Network(prefix) => {
                write!(f, "{:?}", prefix)
            },
        }
    }
}
