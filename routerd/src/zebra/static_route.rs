//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Static route.
//

use std::io;
use std::rc::Rc;
//use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use rtable::tree::*;
use rtable::prefix::*;

use crate::core::config::*;

/// IPv4 Static route configs.
pub struct Ipv4StaticRoute {
    /// Config.
    config: Tree<Prefix<Ipv4Addr>, Rc<StaticRoute<Ipv4Addr>>>,
}

impl Ipv4StaticRoute {
    /// Constructor.
    pub fn new() -> Ipv4StaticRoute {
        Ipv4StaticRoute {
            config: Tree::new(),
        }
    }
}

impl Config for Ipv4StaticRoute {
    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str {
        "ipv4_routes"
    }

    /// Handle POST method.
    fn post(&self, key: &Key, params: Option<&serde_json::Value>) -> Result<(), io::Error> {
        match params {
            Some(json) => {
            },
            None => {
            }
        }

        Ok(())
    }
}


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
