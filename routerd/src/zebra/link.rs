//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - Link handler.
//

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::net::{Ipv4Addr, Ipv6Addr};

use log::error;

use super::address::*;
use super::error::*;
use super::kernel::KernelLink;

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

/// Link Master.
pub struct LinkMaster {

    /// Ifindex to Link map.
    index_map: RefCell<HashMap<i32, Rc<Link>>>,

    /// TBD
    name_map: RefCell<HashMap<String, Rc<Link>>>,
}

impl LinkMaster {

    /// Constructor.
    pub fn new() -> LinkMaster {
        LinkMaster {
            index_map: RefCell::new(HashMap::new()),
            name_map: RefCell::new(HashMap::new()),
        }
    }

    /// Add link to table.
    pub fn add_link(&mut self, link: Link) {
        let link = Rc::new(link);

        self.index_map.borrow_mut().insert(link.index(), link.clone());
        self.name_map.borrow_mut().insert(String::from(link.name()), link.clone());
    }

    /// Delete link from tables.
    pub fn delete_link(&mut self, _link: Link) {
        // TBD
    }

    /// Add IPv4 Address to link with the index.
    pub fn add_ipv4_address(&mut self, index: i32, conn: Connected<Ipv4Addr>) {
        match self.index_map.borrow_mut().get(&index) {
            Some(link) => link.add_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Delete IPv4 Address from the link.
    pub fn delete_ipv4_address(&mut self, index: i32, conn: Connected<Ipv4Addr>) {
        match self.index_map.borrow_mut().get(&index) {
            Some(link) => link.delete_ipv4_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Add IPv6 Address to link with the index.
    pub fn add_ipv6_address(&mut self, index: i32, conn: Connected<Ipv6Addr>) {
        match self.index_map.borrow_mut().get(&index) {
            Some(link) => link.add_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }

    /// Delete IPv6 Address from the link.
    pub fn delete_ipv6_address(&mut self, index: i32, conn: Connected<Ipv6Addr>) {
        match self.index_map.borrow_mut().get(&index) {
            Some(link) => link.delete_ipv6_address(conn),
            None => error!("No link found with index {}", index),
        }
    }
}

/// Link.
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
    addr4: RefCell<Vec<Connected<Ipv4Addr>>>,
    addr6: RefCell<Vec<Connected<Ipv6Addr>>>,
}

impl Link {

    /// Constructor.
    pub fn new(index: i32, name: &str, hwtype: u16, hwaddr: [u8; 6], mtu: u32) -> Link {
        Link {
            index,
            name: name.to_string(),
            hwtype: hwtype,
            hwaddr: hwaddr,
            mtu: mtu,
            addr4: RefCell::new(Vec::new()),
            addr6: RefCell::new(Vec::new()),
        }
    }

    /// Construct from KernelLink.
    pub fn from_kernel(kl: KernelLink) -> Link {
        Link {
            index: kl.ifindex,
            hwtype: kl.hwtype,
            name: kl.name,
            hwaddr: kl.hwaddr,
            mtu: kl.mtu,
            addr4: RefCell::new(Vec::new()),
            addr6: RefCell::new(Vec::new()),
        }
    }

    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
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

