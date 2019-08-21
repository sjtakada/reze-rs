//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Link handler
//

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr};

use super::master::*;
use super::address::*;

/// Abstracted event handler between Zebra and OS.
pub trait LinkHandler {
    /// Get all links from kernel.
    fn get_links_all(&self) -> Result<(), io::Error>;

    /// Set MTU.
    fn set_mtu(&self, mtu: u16) -> bool; // ? Error

    /// Set link up.
    fn set_link_up(&self) -> bool;

    /// Set link down.
    fn set_link_down(&self) -> bool;
}

/// Generic Link information
pub struct Link {
    /// Interface index.
    index: i32,

    /// Name from kernel.
    name: String,
    
    /// Hardware type.
    hwtype: u16,

    /// Hardware address.
    hwaddr: [u8; 6],

    /// MTU.
    mtu: u32,

    /// Connected addresses.
    addr4: Vec<Connected<Ipv4Addr>>,
    addr6: Vec<Connected<Ipv6Addr>>,
}

impl Link {
    pub fn new(index: i32, name: &str, hwtype: u16, hwaddr: [u8; 6], mtu: u32) -> Link {
        Link {
            index,
            hwtype,
            name: name.to_string(),
            hwaddr,
            mtu,
            addr4: Vec::new(),
            addr6: Vec::new(),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }
}

