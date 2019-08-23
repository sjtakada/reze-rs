//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Static route.
//

//use std::net::{Ipv4Addr, Ipv6Addr};

//use rtable::tree::*;
use rtable::prefix::*;

/// Static route.
pub struct StaticRoute<T> {
    /// Prefix.
    prefix: Prefix<T>,

    /// Administrative distance.
    distance: u8,

    /// Route tag.
    tag: u32,

    /// Nexthop(s).
    nexthops: Vec<Nexthop<T>>,
}

impl<T: Clone> StaticRoute<T> {
    pub fn new(prefix: Prefix<T>, distance: u8, tag: u32) -> StaticRoute<T> {
        StaticRoute {
            prefix,
            distance,
            tag,
            nexthops: Vec::new(),
        }
    }

    pub fn prefix(&self) -> &Prefix<T> {
        &self.prefix
    }

    pub fn distance(&self) -> u8 {
        self.distance
    }

    pub fn tag(&self) -> u32 {
        self.tag
    }

    pub fn add_nexthop_ip(&mut self, address: &T) {
        let nexthop = Nexthop::from_address(address);

        self.nexthops.push(nexthop);
    }
}


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
