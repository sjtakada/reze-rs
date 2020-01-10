//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use std::rc::Weak;
use std::str::FromStr;
use std::hash::Hash;
use std::collections::HashMap;

use log::debug;

use rtable::prefix::*;
use rtable::tree::*;

use super::master::*;
use super::nexthop::*;
use super::static_route::*;

/// RIB type.
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

/// RIB.
pub struct Rib<T: Addressable> {
    /// Type.
    _rib_type: RibType,

    /// Nexthops.
    nexthops: Vec<Nexthop<T>>,

    /// Administrative distance.
    distance: u8,

    /// Time updated.
    _instant: time::Instant,

    /// Tag -- TBD placeholder.
    _tag: u32,
}

impl<T> Rib<T>
where T: Addressable + Clone + FromStr + Eq + Hash
{
    /// Constructor.
    pub fn new(rib_type: RibType, distance: u8) -> Rib<T> {
        Rib {
            _rib_type: rib_type,
            nexthops: Vec::new(),
            distance: distance,
            _instant: time::Instant::now(),
            _tag: 0,
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

    pub fn nexthops(&self) -> &Vec<Nexthop<T>> {
        &self.nexthops
    }

    pub fn add_nexthop(&mut self, nexthop: Nexthop<T>) {
        self.nexthops.push(nexthop);
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
    tree: Tree<Prefix<T>, Vec<Rib<T>>>,
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

    pub fn add(&mut self, prefix: &Prefix<T>, rib: Rib<T>) {
        let v = Vec::new();
        if let Some(_) = self.tree.insert(prefix, v) {
            debug!("Prefix {:?} exists", prefix);
        }

        debug!("rib add {:?}", prefix);

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            // TBD: compare existing RIB with the same type, and replace it if they are different
            // and then run RIB update process.

            if let Some(master) = self.master.upgrade() {
                master.rib_install_kernel(prefix, &rib);
            }

            // TBD: we should do route selection here or somewhere else.
            match *node.data() {
                Some(ref mut v) => {
                    v.push(rib);
                }
                None => {}
            }
        }
    }

    pub fn delete(&mut self, prefix: &Prefix<T>) {
        debug!("rib delete {:?}", prefix);

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            // TBD: compare existing RIB with the same type, and replace it if they are different
            // and then run RIB update process.

            // TBD: we should do route selection here.
            if let Some(master) = self.master.upgrade() {
                match *node.data() {
                    Some(ref mut v) => {
                        let rib = v.iter().next().unwrap();

                        master.rib_uninstall_kernel(prefix, &rib);

//                        v.drain();
                    },
                    None => {},
                }
            }
        }
    }
}
