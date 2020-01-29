//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use std::fmt::Debug;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use log::debug;

use rtable::prefix::*;
use rtable::tree::*;

use common::error::*;

use super::master::*;
use super::nexthop::*;
use super::static_route::*;
use super::super::core::mds::*;

/// RIB type.
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub enum RibType {
    System,
    Kernel,
    Connected,
    Static,
    Eigrp,
    Ospf,
    Isis,
    Rip,
    Bgp,
}

/// RIB, store essential routing information with nexthops per single protocol type.
pub struct Rib<T: Addressable>
{
    /// Type.
    rib_type: RibType,

    /// Administrative distance.
    distance: u8,

    /// Time updated.
    instant: time::Instant,

    /// Tag -- TBD placeholder.
    _tag: u32,

    /// TBD: Flag selected.
    selected: Cell<bool>,

    /// TBD: Flag FIB. 
    fib: Cell<bool>,

    /// Nexthops.
    nexthops: RefCell<Vec<Nexthop<T>>>,
}

impl<T> Clone for Rib<T>
where T: Addressable
{
    fn clone(&self) -> Self {
        Rib::<T> {
            rib_type: self.rib_type,
            distance: self.distance,
            instant: self.instant,
            _tag: self._tag,
            selected: Cell::new(self.selected.get()),
            fib: Cell::new(self.fib.get()),
            nexthops: RefCell::new(self.nexthops.borrow().to_vec()),
        }
    }
}

impl<T> Rib<T>
where T: Addressable
{
    /// Constructor.
    pub fn new(rib_type: RibType, distance: u8) -> Rib<T> {
        Rib {
            rib_type: rib_type,
            distance: distance,
            instant: time::Instant::now(),
            _tag: 0,
            selected: Cell::new(false),
            fib: Cell::new(false),
            nexthops: RefCell::new(Vec::new()),
        }
    }

    /// Construct RIB from static route config.
    /// Classify static routes by distance, may return multiple RIBs.
    pub fn from_static_route(sr: Arc<StaticRoute<T>>) -> HashMap<u8, Rib<T>> {
        let mut map = HashMap::<u8, Rib<T>>::new();

        for (nexthop, info) in sr.nexthops().iter() {
            let distance = info.distance();

            let rib = match map.get_mut(&distance) {
                Some(rib) => rib,
                None => {
                    let rib = Rib::<T>::new(RibType::Static, distance);
                    map.insert(distance, rib);
                    map.get_mut(&distance).unwrap()
                }
            };

            rib.add_nexthop(nexthop.clone());
        }

        map
    }

    pub fn key(&self) -> RibKey {
        (self.distance, self.rib_type)
    }

    pub fn rib_type(&self) -> RibType {
        self.rib_type
    }

    pub fn distance(&self) -> u8 {
        self.distance
    }

    pub fn uptime(&self) -> time::Duration {
        time::Instant::now() - self.instant
    }

    pub fn set_selected(&self, selected: bool) {
        self.selected.set(selected)
    }

    pub fn set_fib(&self, fib: bool) {
        self.fib.set(fib)
    }

    pub fn nexthops(&self) -> RefMut<Vec<Nexthop<T>>> {
        self.nexthops.borrow_mut()
    }

    pub fn add_nexthop(&self, nexthop: Nexthop<T>) {
        self.nexthops.borrow_mut().push(nexthop);
    }

    pub fn delete_nexthop(&self, nexthop: &Nexthop<T>) {
        let nexthops = self.nexthops.replace(Vec::new());
        for nh in nexthops {
            if nh != *nexthop {
                self.nexthops.borrow_mut().push(nh);
            }
        }
    }
}

/// RIB candidate key.
///
type RibKey = (u8, RibType);

/// RIB entry.
///   Store a selected FIB, as well as all candidates per RibKey (distance, RibType).
///
pub struct RibEntry<T: Addressable>
{
    /// FIB.
    fib: RefCell<Option<Rib<T>>>,

    /// Candidate RIBs.
    ribs: RefCell<BTreeMap<RibKey, Rib<T>>>,

    /// Updated.
    updated: Cell<bool>,
}

impl<T> RibEntry<T>
where T: Addressable
{
    /// Constructor.
    pub fn new() -> RibEntry<T> {
        RibEntry {
            fib: RefCell::new(None),
            ribs: RefCell::new(BTreeMap::new()),
            updated: Cell::new(false),
        }
    }

    /// FIB.
    pub fn fib(&self) -> RefMut<Option<Rib<T>>> {
        self.fib.borrow_mut()
    }

    /// FIB type.
    pub fn fib_type(&self) -> Option<RibType> {
        match self.fib.borrow().as_ref() {
            Some(fib) => Some(fib.rib_type),
            None => None,
        }
    }

    /// Candidates.
    pub fn ribs(&self) -> RefMut<BTreeMap<RibKey, Rib<T>>> {
        self.ribs.borrow_mut()
    }

    /// Is any of RIBs updated?
    pub fn is_updated(&self) -> bool {
        self.updated.get()
    }

    /// Select RIB.
    /// TBD: Right now select head of candidate RIBs.
    ///      We have to do nexthop activate check.
    pub fn select(&self) -> Option<Rib<T>> {
        if let Some((_, rib)) = self.ribs().iter().next() {
            Some(rib.clone())
        } else {
            None
        }
    }
}


/// RIB table.
///   Each RIB entry is indexed by prefix (address + prefix length). Multiple RIB entries may
///   be stored in each prefix, per different protocol type and distance.
///
pub struct RibTable<T: Addressable>
{
    /// Table tree.
    tree: Tree<Prefix<T>, Rc<RibEntry<T>>>,
}

impl<T> RibTable<T>
where T: Addressable
{
    /// Constructor.
    pub fn new() -> RibTable<T> {
        RibTable {
            tree: Tree::new(),
        }
    }

    /// Add given RIBs into tree per prefix.
    pub fn add(&mut self, prefix: &Prefix<T>, rib: Rib<T>) {
        debug!("rib add {:?} type {:?} distance {:?}", prefix, rib.rib_type(), rib.distance());

        // Create RIB entry if it doesn't exist.
        let it = self.tree.get_node_ctor(prefix, Some(|| { Rc::new(RibEntry::new()) }));

        if let Some(ref node) = *it.node() {
            // Node data must be present.
            if let Some(ref mut entry) = *node.data() {
                entry.ribs().insert(rib.key(), rib);
            }
        }
    }

    /// Delete given RIBs from tree per prefix.
    pub fn delete(&mut self, prefix: &Prefix<T>, rib: Rib<T>) {
        debug!("rib delete {:?} type {:?} distance {:?}", prefix, rib.rib_type(), rib.distance());

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            if let Some(ref mut entry) = *node.data() {
                if let Some(rib_old) = entry.ribs().get(&rib.key()) {
                    for nh in rib.nexthops().iter() {
                        rib_old.delete_nexthop(&nh);
                    }
                }
            }
        }
    }

    /// Process route selection algorithm per prefix.
    /// Core part of decision mechanism, including nexthop activity check.
    pub fn process<F>(&mut self, prefix: &Prefix<T>, kfunc: F)
    where F: Fn(&Prefix<T>, &RibEntry<T>) -> Option<Rib<T>>
    {
        debug!("rib process {:?}", prefix);

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            if let Some (ref mut entry) = *node.data() {
                let mut to_be_removed = Vec::new();

                for (key, rib) in entry.ribs().iter() {
                    if rib.nexthops().len() == 0 {
                        to_be_removed.push(key.clone());
                    }
                }

                for key in to_be_removed {
                    entry.ribs().remove(&key);
                }

                // Call kernel function to install/unistall selected route.
                if let Some(fib) = kfunc(prefix, entry) {
                    entry.fib().replace(fib);
                } else {
                    entry.fib().take();
                }
            }
        }
    }

    /// Lookup RIB entry per prefix.
    pub fn lookup_exact(&self, prefix: &Prefix<T>) -> Option<Rc<RibEntry<T>>> {
        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            node.data().clone()
        } else {
            None
        }
    }

    /// Dump rib into JSON
//    pub fn to_sjon(&self) -> serde_json::Value {
//        let mut v = serde_json::Value::new();
//
//        v
//    }

    /// Dump route in json string
    pub fn to_string(&self) -> String {
        let mut v = Vec::new();

        for node in self.tree.into_iter() {
            v.push(node.prefix().to_string());
        }

        format!("{:?}", v)
    }
}

/// Wrapper Ipv4 Rib Table.
pub struct RibTableIpv4 {

    /// Zebra master.
    master: Rc<ZebraMaster>,

    count: Cell<u32>,
}

impl RibTableIpv4 {

    /// Constructor.
    pub fn new(master: Rc<ZebraMaster>) -> RibTableIpv4 {
        RibTableIpv4 {
            master: master,
            count: Cell::new(0),
        }
    }
}

/// MdsHandler implementation for RibTable<Ipv4Addr>
impl MdsHandler for RibTableIpv4 {

    /// Handle GET method.
    fn handle_get(&self, _path: &str, _params: Option<Box<String>>) -> Result<Option<String>, CoreError> {
        let master = self.master.clone();

        let mut rib_ipv4 = master.rib_ipv4();
        let s = rib_ipv4.to_string();

        debug!("*** handle get rib table {}", s);

        let c = self.count.get();
        self.count.set(c + 1);

        Ok(Some(format!("RIB output {}\n", c)))
    }
}

///
/// Unit tests for RIB.
///
#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
//use std::net::Ipv6Addr;
    use super::*;

    #[test]
    pub fn test_rib_process() {
        let mut table = RibTable::<Ipv4Addr>::new();
        let rib1 = Rib::<Ipv4Addr>::new(RibType::Static, 1);
        let rib2 = Rib::<Ipv4Addr>::new(RibType::Static, 200);
        let rib3 = Rib::<Ipv4Addr>::new(RibType::Ospf, 110);
        let p = Prefix::<Ipv4Addr>::from_str("10.10.10.0/24").unwrap();
        let addr = "1.1.1.1".parse().unwrap();
        let nh = Nexthop::<Ipv4Addr>::from_address(&addr);

        rib1.add_nexthop(nh.clone());
        rib2.add_nexthop(nh.clone());
        rib3.add_nexthop(nh.clone());

        let rib1_clone = rib1.clone();

        table.add(&p, rib1);
        let entry = table.lookup_exact(&p).unwrap();
        if let Some(ref mut _fib) = *entry.clone().fib() {
            assert!(false);
        }

        {
            table.process(&p, |_, _| { entry.select() });
            let fib = entry.fib();
            assert_eq!(fib.is_some(), true);
            assert_eq!((*fib).as_ref().unwrap().rib_type(), RibType::Static);
        }

        {
            table.add(&p, rib2);
            table.process(&p, |_, _| { entry.select() });
            let fib = entry.fib();
            assert_eq!(fib.is_some(), true);
            assert_eq!((*fib).as_ref().unwrap().rib_type(), RibType::Static);
        }

        {
            table.add(&p, rib3);
            table.process(&p, |_, _| { entry.select() });
            let fib = entry.fib();
            assert_eq!(fib.is_some(), true);
            assert_eq!((*fib).as_ref().unwrap().rib_type(), RibType::Static);
        }

        {
            table.delete(&p, rib1_clone);
            table.process(&p, |_, _| { entry.select() });
            assert_eq!(entry.fib_type(), Some(RibType::Ospf));
        }
    }
}
