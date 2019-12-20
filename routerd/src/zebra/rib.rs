//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use std::fmt;
use std::rc::Rc;
use std::rc::Weak;

use log::debug;

use rtable::prefix::*;
use rtable::tree::*;

use super::master::*;
use super::nexthop::*;

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
pub struct Rib<T: AddressLen> {
    /// Type.
    _rib_type: RibType,

    /// Nexthops.
    nexthops: Vec<Nexthop<T>>,

    /// Tag.
    _tag: u32,

    /// Administrative distance.
    _distance: u8,

    /// Time updated.
    _instant: time::Instant,
}

impl<T: AddressLen> Rib<T> {
    pub fn new(rib_type: RibType, distance: u8, tag: u32) -> Rib<T> {
        Rib {
            _rib_type: rib_type,
            nexthops: Vec::new(),
            _tag: tag,
            _distance: distance,
            _instant: time::Instant::now(),
        }
    }

    pub fn nexthops(&self) -> &Vec<Nexthop<T>> {
        &self.nexthops
    }

    pub fn add_nexthop(&mut self, nexthop: Nexthop<T>) {
        self.nexthops.push(nexthop);
    }
}

/// RIB table.
pub struct RibTable<T: AddressLen + Clone> {
    /// Zebra master.
    master: Weak<ZebraMaster>,

    /// Table tree.
    tree: Tree<Prefix<T>, Vec<Rib<T>>>,
}

impl<T: AddressLen + Clone + fmt::Debug> RibTable<T> {
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

    pub fn add(&mut self, rib: Rib<T>, prefix: &Prefix<T>) {
        let v = Vec::new();
        if let Some(_) = self.tree.insert(prefix, v) {
            debug!("Prefix {:?} exists", prefix);
        }

        debug!("rib add");

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            // TBD: compare existing RIB with the same type, and replace it if they are different
            // and then run RIB update process.


            if let Some(master) = self.master.upgrade() {
                master.rib_install_kernel(prefix, &rib);
            }

            match *node.data() {
                Some(ref mut v) => {
                    v.push(rib);
                }
                None => {}
            }
        }
    }
}
