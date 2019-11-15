//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Nexthop.
//

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
