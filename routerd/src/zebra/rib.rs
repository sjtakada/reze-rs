//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use std::fmt;

use log::debug;

use rtable::prefix::*;
use rtable::tree::*;

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

/// RIB
pub struct Rib<P: Prefixable> {
    /// Type.
    rib_type: RibType,

    /// Nexthops.
    nexthops: Vec<Nexthop<P>>,

    /// Tag.
    tag: u32,

    /// Time updated.
    instant: time::Instant,
}

impl<P: Prefixable> Rib<P> {
    pub fn new(rib_type: RibType, distance: u8, tag: u32) -> Rib<P> {
        Rib {
            rib_type: rib_type,
            nexthops: Vec::new(),
            tag: tag,
            instant: time::Instant::now(),
        }
    }

    pub fn add_nexthop(&mut self, nexthop: Nexthop<P>) {
        self.nexthops.push(nexthop);
    }
}

/// RIB table.
pub struct RibTable<P: Prefixable> {
    /// Table tree.
    tree: Tree<P, Vec<Rib<P>>>,
}

impl<P: Prefixable + fmt::Debug> RibTable<P> {
    pub fn new() -> RibTable<P> {
        RibTable {
            tree: Tree::new(),
        }
    }

    pub fn add(&mut self, rib: Rib<P>, prefix: &P) {
        let v = Vec::new();
        if let Some(_) = self.tree.insert(prefix, v) {
            debug!("Prefix {:?} exists", prefix);
        }

        debug!("rib add");

        let it = self.tree.lookup_exact(prefix);
        if let Some(ref node) = *it.node() {
            // TBD: compare existing RIB with the same type, and replace it if they are different
            // and then run RIB update process.

            match *node.data() {
                Some(ref mut v) => {
                    v.push(rib);
                }
                None => {}
            }
        }
    }
}
