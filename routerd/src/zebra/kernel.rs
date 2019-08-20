//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel interface
//

use std::cell::Cell;

use super::master::*;
use super::link::*;
use super::address::*;

use super::linux::netlink::*;

/// Kernel interface.
pub struct Kernel {
    // Zebra Master.
    //master: Cell<Option<&'a ZebraMaster<'a>>>,

    /// Netlink socket.
    netlink: Netlink,
}

impl Kernel {
    pub fn new() -> Kernel {
        let netlink = Netlink::new().unwrap();

        Kernel {
            netlink
        }
    }

    pub fn init(&self) {
println!("*** init 00");
//        self.master = Some(master);

        let links = self.netlink.get_links_all().unwrap();

        for l in links {
            println!("*** ifindex={}, name={}, hwaddr={:?}, mtu={}", l.index, l.name, l.hwaddr, l.mtu);
        }

println!("*** init 20");
        let v4addr = self.netlink.get_ipv4_addresses_all();
println!("*** init 30");
        let v6addr = self.netlink.get_ipv6_addresses_all();
println!("*** init 99");
    }
}
