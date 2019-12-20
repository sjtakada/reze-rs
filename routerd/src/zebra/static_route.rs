//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Static route.
//

use std::rc::Rc;
use std::sync::Arc;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
//use std::net::Ipv6Addr;
use std::str::FromStr;

use serde_json;

use log::debug;
use rtable::prefix::*;

use crate::core::protocols::ProtocolType;
use crate::core::config::*;
use crate::core::error::*;
use super::master::ZebraMaster;
use super::nexthop::*;

/// IPv4 Static route configs.
pub struct Ipv4StaticRoute {
    /// Zebra master.
    master: Rc<ZebraMaster>,

    /// Config.
    _config: BTreeMap<Prefix<Ipv4Addr>, Arc<StaticRoute<Ipv4Addr>>>,
}

impl Ipv4StaticRoute {
    /// Constructor.
    pub fn new(master: Rc<ZebraMaster>) -> Ipv4StaticRoute {
        Ipv4StaticRoute {
            master: master,
            _config: BTreeMap::new(),
        }
    }
}

impl Config for Ipv4StaticRoute {
    /// Return unique identifier, this is used to register to parent as a key.
    fn id(&self) -> &str {
        "route_ipv4"
    }

    /// Handle POST method.
    fn post(&self, path: &str, params: Option<Box<String>>) -> Result<(), CoreError> {
        match params {
            Some(json_str) => {
                debug!("Configuring IPv4 static routes");

                match split_id_and_path(path) {
                    Some((addr_str, none_or_mask_str)) => {
                        // TODO: should handle error.
                        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

                        match none_or_mask_str {
                            Some(mask_str) => {
                                // Trim leading "/" from mask_str.
                                self.master.rib_add_static_ipv4(&addr_str, &mask_str[1..], &json);
                            },
                            None => {
                                self.master.rib_add_static_ipv4(&addr_str, "255.255.255.255", &json);
                            }
                        };
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
pub struct StaticRoute<T: AddressLen> {
    /// Prefix.
    prefix: Prefix<T>,

    /// Administrative distance.
    distance: u8,

    /// Route tag.
    tag: u32,

    /// Nexthop(s).
    nexthops: Vec<Nexthop<T>>,
}

impl<T: Clone + AddressLen + FromStr> StaticRoute<T> {
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

    pub fn add_nexthop_address(&mut self, address: &T) {
        let nexthop = Nexthop::from_address(address);

        self.nexthops.push(nexthop);
    }
}
