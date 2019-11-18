//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Nexthop.
//

use std::fmt;
use rtable::prefix::*;

/// Nexthop.
pub enum Nexthop<T> {
    /// IP Address.
    Address(T),

    /// Interface Name.
    Ifname(String),

    /// Network Prefix - TBD: floating nexthop.
    Network(Prefix<T>),
}

impl<T: Clone + AddressLen> Nexthop<T> {
    /// Construct Nexthop from IP address.
    pub fn from_address(address: &T) -> Nexthop<T> {
        Nexthop::<T>::Address(address.clone())
    }

    /// Construct Nexthop from Interface name.
    pub fn from_ifname(ifname: &str) -> Nexthop<T> {
        Nexthop::<T>::Ifname(String::from(ifname))
    }
}

impl<T: fmt::Debug> fmt::Display for Nexthop<T> {
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
