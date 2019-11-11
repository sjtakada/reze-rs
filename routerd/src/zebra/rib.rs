//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra RIB - Routing Information Base
//

use std::time;
use rtable::prefix::*;
use super::nexthop::*;

pub enum RibType {
    System,
    Kernel,
    Connected,
    Static,
    Rip,
    Eigrp,
    Ospf,
    Isis,
    Bgp,
}

/// RIB
pub struct Rib<T> {
    /// Nexthops.
    nexthops: Vec<Nexthop<T>>,

    /// Tag.
    tag: u32,

    /// Time updated.
    instant: time::Instant,
}

impl<T: Clone> Rib<T> {
    pub fn new(prefix: Prefix<T>, distance: u8, tag: u32) -> Rib<T> {
        Rib {
            nexthops: Vec::new(),
            tag: tag,
            instant: time::Instant::now(),
        }
    }
}
