//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Static route.
//

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
//use std::net::Ipv6Addr;
use std::str::FromStr;

use serde_json;
use log::{debug, error};
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
    config: RefCell<BTreeMap<Prefix<Ipv4Addr>, Arc<StaticRoute<Ipv4Addr>>>>,
}

impl Ipv4StaticRoute {
    /// Constructor.
    pub fn new(master: Rc<ZebraMaster>) -> Ipv4StaticRoute {
        Ipv4StaticRoute {
            master: master,
            config: RefCell::new(BTreeMap::new()),
        }
    }

    /// Add a static route config into the tree.
    pub fn add(&self, p: Prefix<Ipv4Addr>, s: Arc<StaticRoute<Ipv4Addr>>) -> Option<Arc<StaticRoute<Ipv4Addr>>> {
        self.config.borrow_mut().insert(p, s)
    }

    // Delete a static route config into the tree.
//    pub fn delete(&mut self, p: Prefix<Ipv4Addr>, s: Arc<StaticRoute<Ipv4Addr>>) {
//        self.config.delete(&p);
//    }
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
                                if let Ok(prefix) = prefix_ipv4_from(&addr_str, &mask_str[1..]) {
                                    let config = Arc::new(StaticRoute::<Ipv4Addr>::from_json(&prefix, &json));
                                    let _config_old = self.add(prefix, config.clone());

                                    self.master.rib_add_static_ipv4(config);
//                                self.master.rib_add_static_ipv4(&addr_str, &mask_str[1..], &json);
                                } else {
                                    debug!("Invalid address or mask {} {}", addr_str, mask_str);
                                }
                            },
                            None => {
//                                self.master.rib_add_static_ipv4(&addr_str, "255.255.255.255", &json);
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

    pub fn from_json(prefix: &Prefix<T>, params: &serde_json::Value) -> StaticRoute<T> {
        let mut distance = 1u8;
        let mut tag = 0u32;
        let mut nexthops: Vec<Nexthop<T>> = Vec::new();

        if params.is_object() {
            for key in params.as_object().unwrap().keys() {
                match key.as_ref() {
                    "distance" => {
                        if params[key].is_number() {
                            distance = params[key].as_u64().unwrap() as u8;
                        }
                    },
                    "tag" => {
                        if params[key].is_number() {
                            tag = params[key].as_u64().unwrap() as u32;
                        }
                    },
                    "nexthops" => {
                        if params[key].is_array() {
                            for nexthop in params[key].as_array().unwrap() {
                                for t in nexthop.as_object().unwrap().keys() {
                                    match t.as_ref() {
                                        "ipv4_address" => {
                                            match Nexthop::<T>::from_address_str(nexthop[t].as_str().unwrap()) {
                                                Some(address) => nexthops.push(address),
                                                None => {}
                                            }
                                        },
                                        "interface" => {
                                        },
                                        "ipv4_network" => {
                                        },
                                        _ => {
                                            error!("Unknown nexthop type {}", t)
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {
                        error!("Unknown static route param {}", key);
                    }
                }
            }
        }

        StaticRoute {
            prefix: prefix.clone(),
            distance: distance,
            tag: tag,
            nexthops: nexthops,
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

    pub fn nexthops(&self) -> &Vec<Nexthop<T>> {
        &self.nexthops
    }

    pub fn add_nexthop_address(&mut self, address: &T) {
        let nexthop = Nexthop::from_address(address);

        self.nexthops.push(nexthop);
    }
}

impl<T: AddressLen + PartialEq> PartialEq for StaticRoute<T> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix
    }
}

impl<T: AddressLen + Eq> Eq for StaticRoute<T> {}


impl<T: AddressLen + PartialOrd> PartialOrd for StaticRoute<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.prefix.partial_cmp(&other.prefix)
    }
}

impl<T: AddressLen + Ord> Ord for StaticRoute<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.prefix.cmp(&other.prefix)
    }
}


///
/// Unit tests for StaticRoute.
///
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_ipv4_static_route_cmp() {
        let p1 = Prefix::<Ipv4Addr>::from_str("10.0.0.0/24").unwrap();
        let p2 = Prefix::<Ipv4Addr>::from_str("10.0.0.0/16").unwrap();
        let p3 = Prefix::<Ipv4Addr>::from_str("10.10.0.0/24").unwrap();

        assert_eq!(p1 > p2, true);
        assert_eq!(p1 < p3, true);
        assert_eq!(p2 < p3, true);

        let s1 = StaticRoute::<Ipv4Addr>::new(p1, 30, 0);
        let s2 = StaticRoute::<Ipv4Addr>::new(p2, 20, 0);
        let s3 = StaticRoute::<Ipv4Addr>::new(p3, 10, 0);

        assert_eq!(s1 > s2, true);
        assert_eq!(s1 < s3, true);
        assert_eq!(s2 < s3, true);
    }
}
