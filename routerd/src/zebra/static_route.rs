//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Static route.
//

use std::io;
use std::rc::Rc;
use std::sync::Arc;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
//use std::net::Ipv6Addr;

use serde_json;

use log::debug;
use rtable::prefix::*;

use crate::core::protocols::ProtocolType;
use crate::core::config::*;
use super::master::ZebraMaster;
use super::nexthop::*;

/// IPv4 Static route configs.
pub struct Ipv4StaticRoute {
    /// 
    master: Rc<ZebraMaster>,

    /// Config.
    config: BTreeMap<Prefix<Ipv4Addr>, Arc<StaticRoute<Ipv4Addr>>>,
}

impl Ipv4StaticRoute {
    /// Constructor.
    pub fn new(master: Rc<ZebraMaster>) -> Ipv4StaticRoute {
        Ipv4StaticRoute {
            master: master,
            config: BTreeMap::new(),
        }
    }
}

impl Config for Ipv4StaticRoute {
    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str {
        "route_ipv4"
    }

    /// Handle POST method.
    fn post(&self, path: &str, params: Option<Box<String>>) -> Result<(), io::Error> {
        match params {
            Some(json_str) => {
                debug!("Configuring IPv4 static routes");

                match split_id_and_path(path) {
                    Some((addr, opt_mask)) => {
                        let mask = opt_mask.unwrap_or("255.255.255.255".to_string());

                        // TODO: should handle error.
                        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

                        self.master.rib_add_static_ipv4(&addr, &mask, &json);
                    },
                    None => {
                        debug!("Invalid path");
                    }
                }
            },
            None => {
                debug!("No parameters")
            },
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
