//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Link handler
//

use std::cell::RefCell;
use std::net::{Ipv4Addr, Ipv6Addr};

//use super::master::*;
use super::address::*;
use super::error::*;

/// Abstracted event handler between Zebra and OS.
pub trait LinkHandler {
    /// Get all links from kernel.
    fn get_links_all(&self) -> Result<(), ZebraError>;

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
    pub name: String,
    
    /// Hardware type.
    hwtype: u16,

    /// Hardware address.
    hwaddr: [u8; 6],

    /// MTU.
    mtu: u32,

    /// Connected addresses.
    addr4: RefCell<Vec<Connected<Ipv4Addr>>>,
    addr6: RefCell<Vec<Connected<Ipv6Addr>>>,
}

impl Link {
    pub fn new(index: i32, name: &str, hwtype: u16, hwaddr: [u8; 6], mtu: u32) -> Link {
        Link {
            index,
            hwtype: hwtype,
            name: name.to_string(),
            hwaddr: hwaddr,
            mtu: mtu,
            addr4: RefCell::new(Vec::new()),
            addr6: RefCell::new(Vec::new()),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn hwtype(&self) -> u16 {
        self.hwtype
    }

    pub fn hwaddr(&self) -> &[u8; 6] {
        &self.hwaddr
    }

    pub fn mtu(&self) -> u32 {
        self.mtu
    }

    pub fn add_ipv4_address(&self, conn: Connected<Ipv4Addr>) {
        self.addr4.borrow_mut().push(conn);
    }

    pub fn delete_ipv4_address(&self, conn: Connected<Ipv4Addr>) {
        let len = self.addr4.borrow().len();
        let list = self.addr4.borrow_mut();

        for i in 0..len {
            let c = &list[i];
            if c.address() == conn.address() {
                self.addr4.borrow_mut().remove(i);
                break;
            }
        }
    }

    pub fn add_ipv6_address(&self, conn: Connected<Ipv6Addr>) {
        self.addr6.borrow_mut().push(conn);
    }

    pub fn delete_ipv6_address(&self, conn: Connected<Ipv6Addr>) {
        let len = self.addr6.borrow().len();
        let list = self.addr6.borrow_mut();

        for i in 0..len {
            let c = &list[i];
            if c.address() == conn.address() {
                self.addr6.borrow_mut().remove(i);
                break;
            }
        }
    }
}

