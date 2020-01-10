//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use std::fmt;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::rc::Rc;
use std::sync::Arc;
use std::rc::Weak;
use std::str::FromStr;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::BTreeMap;

use log::debug;

use rtable::prefix::*;
use rtable::tree::*;

use super::master::*;
use super::nexthop::*;
use super::static_route::*;

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
#[derive(Clone)]
pub struct Rib<T: Addressable> {
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

impl<T> Rib<T>
where T: Addressable + Clone + FromStr + Eq + Hash
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
pub struct RibEntry<T: Addressable> {

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

    /// Candidates.
    pub fn ribs(&self) -> RefMut<BTreeMap<RibKey, Rib<T>>> {
        self.ribs.borrow_mut()
    }

    /// Is any of RIBs updated?
    pub fn is_updated(&self) -> bool {
        self.updated.get()
    }
}


/// RIB table.
///   Each RIB entry is indexed by prefix (address + prefix length). Multiple RIB entries may
///   be stored in each prefix, per different protocol type and distance.
///
pub struct RibTable<T: Addressable + Clone> {

    /// Zebra master.
    master: Weak<ZebraMaster>,

    /// Table tree.
    //tree: Tree<Prefix<T>, Vec<Rib<T>>>,
    tree: Tree<Prefix<T>, RibEntry<T>>,
}

impl<T> RibTable<T>
where T: Addressable + Clone + FromStr + Hash + Eq + fmt::Debug
{
    /// Constructor.
    pub fn new() -> RibTable<T> {
        RibTable {
            master: Default::default(),
            tree: Tree::new(),
        }
    }

    /// Set ZebraMaster.
    pub fn set_master(&mut self, master: Rc<ZebraMaster>) {
        self.master = Rc::downgrade(&master);
    }

    /// Add given RIBs into tree per prefix.
    pub fn add(&mut self, prefix: &Prefix<T>, rib: Rib<T>) {
        debug!("rib add {:?} type {:?} distance {:?}", prefix, rib.rib_type(), rib.distance());

        // Create RIB entry if it doesn't exist.
        let it = self.tree.get_node_ctor(prefix, Some(|| { RibEntry::new() }));

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
    /// Core part of routing mechanism, including nexthop activation check.
    pub fn process(&mut self, prefix: &Prefix<T>) {
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

                if let Some(master) = self.master.upgrade() {
                    if let Some(ref mut fib) = *entry.fib() {
                        master.rib_uninstall_kernel(prefix, &fib);
                    }

                    if let Some((_, selected)) = entry.ribs().iter().next() {
                        master.rib_install_kernel(prefix, &selected);

                        let fib = selected.clone();
                        entry.fib().replace(fib);
                    }
                }
            }
        }
    }
}
