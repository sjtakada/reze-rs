//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Nexthop.
//

use rtable::prefix::*;

/// Nexthop.
pub struct Nexthop<T> {
    /// Name of interface.
    ifname: Option<String>,

    /// IP address.
    address: Option<T>,
    
    /// IP network - TBD.
    _network: Option<Prefix<T>>,
}

impl<T: Clone> Nexthop<T> {
    pub fn from_ifname(ifname: &str) -> Nexthop<T> {
        Nexthop::<T> {
            ifname: Some(String::from(ifname)),
            address: None,
            _network: None,
        }
    }

    pub fn from_address(address: &T) -> Nexthop<T> {
        Nexthop::<T> {
            ifname: None,
            address: Some(address.clone()),
            _network: None
        }
    }

    pub fn ifname(&self) -> Option<&str> {
        match self.ifname {
            Some(ref ifname) => Some(ifname),
            None => None,
        }
    }

    pub fn address(&self) -> Option<&T> {
        match self.address {
            Some(ref address) => Some(address),
            None => None,
        }
    }
}
