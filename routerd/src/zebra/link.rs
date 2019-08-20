//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Link handler
//

use std::io;

// Abstracted event handler between Zebra and OS.
pub trait LinkHandler {
    // Get all links from kernel.
    fn get_links_all(&self) -> Result<Vec<Link>, io::Error>;

    // Add link from zebra
    //fn add_link(&self) -> ?

    // Get link information.
    fn get_link(&self, index: i32) -> Option<Link>;

    // Set MTU.
    fn set_mtu(&self, mtu: u16) -> bool; // ? Error

    // Set link up.
    fn set_link_up(&self) -> bool;

    // Set link down.
    fn set_link_down(&self) -> bool;

    // Set callback for link stat change.
//    fn set_link_change_callback(&self, &Fn());
}

/// Generic Link information
pub struct Link {
    /// Interface index.
    pub index: i32,

    /// Name from kernel.
    pub name: String,
    
    /// Hardware type.
    pub hwtype: u16,

    /// Hardware address.
    pub hwaddr: [u8; 6],

    /// MTU.
    pub mtu: u32,
}

impl Link {
    pub fn new(index: i32, name: &str, hwtype: u16, hwaddr: [u8; 6], mtu: u32) -> Link {
        Link {
            index,
            hwtype,
            name: name.to_string(),
            hwaddr,
            mtu,
        }
    }
}

