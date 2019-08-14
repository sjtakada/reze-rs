//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel interface
//

use super::link::*;
use super::linux::netlink::*;

// Kernel.
pub struct Kernel {
    // Netlink socket.
    netlink: Netlink,
}

impl Kernel {
    pub fn new() -> Kernel {
        Kernel {
            netlink: Netlink::new(),
        }
    }

    pub fn init(&self) {
        let links = self.netlink.get_links_all();

        for l in links {
            println!("*** ifindex={}, name={}, hwaddr={:?}, mtu={}", l.index, l.name, l.hwaddr, l.mtu);
        }

    }
}
