//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - Static route.
//

use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::net::Ipv4Addr;
//use std::net::Ipv6Addr;

use serde_json;
use log::{debug, error};

use rtable::prefix::*;
use common::error::*;
use common::nexthop::*;

use crate::core::mds::*;
use super::master::ZebraMaster;

/// Constants.
const ZEBRA_ADMINISTRATIVE_DISTANCE_DEFAULT: u8 = 1;
const ZEBRA_STATIC_ROUTE_TAG_DEFAULT: u32 = 0;


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

    /// Lookup a static route by prefix.
    pub fn lookup(&self, p: &Prefix<Ipv4Addr>) -> Option<Arc<StaticRoute<Ipv4Addr>>> {
        match self.config.borrow_mut().get(p) {
            Some(sr) => Some(sr.clone()),
            None => None,
        }
    }

    /// Add a static route config into the tree.
    pub fn add(&self, p: Prefix<Ipv4Addr>, sr_new: Arc<StaticRoute<Ipv4Addr>>) -> Arc<StaticRoute<Ipv4Addr>> {
        match self.lookup(&p) {
            Some(sr) => {
                for (nh, info) in sr_new.nexthops.borrow_mut().drain() {
                    sr.nexthops.borrow_mut().insert(nh, info);
                }

                sr.clone()
            },
            None => {
                self.config.borrow_mut().insert(p, sr_new.clone());
                sr_new.clone()
            }
        }
    }

    /// Delete a static route config from the tree.
    pub fn delete(&self, p: Prefix<Ipv4Addr>, sr_new: Arc<StaticRoute<Ipv4Addr>>) -> Arc<StaticRoute<Ipv4Addr>> {
        match self.lookup(&p) {
            Some(sr) => {
                for (nh, _info) in sr_new.nexthops.borrow_mut().iter() {
                    sr.nexthops.borrow_mut().remove(&nh);
                }
            },
            None => {},
        }

        sr_new
    }
}

impl MdsHandler for Ipv4StaticRoute {

    /// Handle PUT method.
    fn handle_put(&self, path: &str, params: Option<Box<String>>) -> Result<Option<String>, CoreError> {
        // TBD: XXXX
        let pat = "/config/route_ipv4";
        if !path.starts_with(pat) {
            return Err(CoreError::CommandExec(format!("Invalid path")));
        }
        let path = &path[pat.len()..];

        match params {
            Some(json_str) => {
                debug!("Configuring an IPv4 static route");

                match split_id_and_path(path) {
                    Some((addr_str, none_or_mask_str)) => {
                        // TODO: should handle error.
                        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
                        let mask_str = match none_or_mask_str {
                            Some(mask_str) => mask_str,
                            None => "/255.255.255.255".to_string(),
                        };

                        // Trim leading "/" from mask_str.
                        if let Ok(prefix) = prefix_ipv4_from(&addr_str, &mask_str[1..]) {
                            let sr_new = Arc::new(StaticRoute::<Ipv4Addr>::from_json(&prefix, &json)?);
                            let sr = self.add(prefix, sr_new);
                            self.master.rib_add_static_ipv4(sr);
                        } else {
                            return Err(CoreError::CommandExec(format!("Invalid address or mask {} {}", addr_str, mask_str)))
                        }
                    },
                    None => {
                        return Err(CoreError::CommandExec(format!("Invalid path")));
                    }
                }
            },
            None => {
                return Err(CoreError::CommandExec(format!("No parameters")));
            },
        }

        Ok(None)
    }

    /// Handle DELETE method.
    fn handle_delete(&self, path: &str, params: Option<Box<String>>) -> Result<Option<String>, CoreError> {
        match params {
            Some(json_str) => {
                debug!("Unconfiguring an IPv4 static route");

                match split_id_and_path(path) {
                    Some((addr_str, none_or_mask_str)) => {
                        // TODO: should handle error.
                        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
                        let mask_str = match none_or_mask_str {
                            Some(mask_str) => mask_str,
                            None => "/255.255.255.255".to_string(),
                        };

                        // Trim leading "/" from mask_str.
                        if let Ok(prefix) = prefix_ipv4_from(&addr_str, &mask_str[1..]) {
                            let sr_new = Arc::new(StaticRoute::<Ipv4Addr>::from_json(&prefix, &json)?);
                            let sr_new = self.delete(prefix.clone(), sr_new);

                            self.master.rib_delete_static_ipv4(sr_new);
                        } else {
                            return Err(CoreError::CommandExec(format!("Invalid address or mask {} {}", addr_str, mask_str)))
                        }
                    },
                    None => {
                        return Err(CoreError::CommandExec(format!("Invalid path")));
                    }
                }
            },
            None => {
                return Err(CoreError::CommandExec(format!("No parameters")));
            },
        }

        Ok(None)
    }
}


/// Static route.
pub struct StaticRoute<T: Addressable> {

    /// Prefix.
    prefix: Prefix<T>,

    /// Nexthop(s).
    nexthops: RefCell<HashMap<Nexthop<T>, StaticRouteInfo>>,
}

impl<T> StaticRoute<T>
where T: Addressable
{
    /// Constructor.
    pub fn new(prefix: Prefix<T>, nexthops: HashMap<Nexthop<T>, StaticRouteInfo>) -> StaticRoute<T> {
        StaticRoute {
            prefix,
            nexthops: RefCell::new(nexthops),
        }
    }

    /// Construct static route from JSON.
    pub fn from_json(prefix: &Prefix<T>, params: &serde_json::Value) -> Result<StaticRoute<T>, CoreError> {
        let mut nexthops: HashMap<Nexthop<T>, StaticRouteInfo> = HashMap::new();

        if !params.is_object() {
            return Err(CoreError::CommandExec("JSON param is not an object".to_string()))
        }

        let params = params.as_object().unwrap();
        if let Some(v_nexthops) = params.get("nexthops") {
            if !v_nexthops.is_array() {
                return Err(CoreError::CommandExec("No nexthop array in params".to_string()))
            }

            for v_nh in v_nexthops.as_array().unwrap() {
                if !v_nh.is_object() {
                    error!("Nexthop is not an object");
                    continue;
                }

                let mut nexthop = None;
                let mut distance = ZEBRA_ADMINISTRATIVE_DISTANCE_DEFAULT;
                let mut tag = ZEBRA_STATIC_ROUTE_TAG_DEFAULT;

                if let Some(nh) = v_nh.get("nexthop") {
                    if nh.is_object() {
                        if let Some(v) = nh.get("ipv4_address") {
                            if let Some(address) = Nexthop::<T>::from_address_str(v.as_str().unwrap()) {
                                nexthop = Some(address.clone());
                            }
                        }

                        if let Some(_v) = nh.get("interface") {
                            // TBD
                        }
                    }
                }

                if let Some(v) = v_nh["distance"].as_u64() {
                    distance = v as u8;
                }

                if let Some(v) = v_nh["tag"].as_u64() {
                    tag = v as u32;
                }

                if let Some(nexthop) = nexthop {
                    nexthops.insert(nexthop, StaticRouteInfo { distance, tag });
                }
            }
        } else {
            return Err(CoreError::CommandExec("No nexthop in params".to_string()))
        }

        if nexthops.len() == 0 {
            return Err(CoreError::CommandExec("No valid nexthops".to_string()))
        }

        Ok(StaticRoute::new(prefix.clone(), nexthops))
    }

    pub fn prefix(&self) -> &Prefix<T> {
        &self.prefix
    }

    pub fn nexthops(&self) -> RefMut<HashMap<Nexthop<T>, StaticRouteInfo>> {
        self.nexthops.borrow_mut()
    }
}

impl<T> PartialEq for StaticRoute<T>
where T: Addressable
{
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix
    }
}

impl<T> Eq for StaticRoute<T>
where T: Addressable
{
}


impl<T> PartialOrd for StaticRoute<T>
where T: Addressable + PartialOrd
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.prefix.partial_cmp(&other.prefix)
    }
}

impl<T> Ord for StaticRoute<T>
where T: Addressable + Ord
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.prefix.cmp(&other.prefix)
    }
}

/// Static route info.
#[derive(Clone)]
pub struct StaticRouteInfo {

    /// Administrative distance.
    distance: u8,

    /// Route tag,
    tag: u32,
}

impl StaticRouteInfo {

    /// Return distance.
    pub fn distance(&self) -> u8 {
        self.distance
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

        let addr = "1.1.1.1".parse().unwrap();
        let nh = Nexthop::<Ipv4Addr>::from_address(&addr);
        let si = StaticRouteInfo { distance: 1, tag: 0 };
        let mut m: HashMap<Nexthop<Ipv4Addr>, StaticRouteInfo> = HashMap::new();
        m.insert(nh, si);

        let s1 = StaticRoute::<Ipv4Addr>::new(p1, m.clone());
        let s2 = StaticRoute::<Ipv4Addr>::new(p2, m.clone());
        let s3 = StaticRoute::<Ipv4Addr>::new(p3, m.clone());

        assert_eq!(s1 > s2, true);
        assert_eq!(s1 < s3, true);
        assert_eq!(s2 < s3, true);
    }
}
