//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Kernel interface
//

use super::link::*;
use super::address::*;

use super::linux::rt_netlink::*;

// Kernel.
pub struct Kernel {
    // Netlink socket.
    rt_netlink: RtNetlink,
}

impl Kernel {
    pub fn new() -> Kernel {
        Kernel {
            rt_netlink: RtNetlink::new(),
        }
    }

    pub fn init(&self) {
println!("*** init 00");
        let links = self.netlink.get_links_all();

        for l in links {
            println!("*** ifindex={}, name={}, hwaddr={:?}, mtu={}", l.index, l.name, l.hwaddr, l.mtu);
        }

println!("*** init 20");
        let v4addr = self.netlink.get_addresses_all(RtAddrFamily::Inet);
println!("*** init 30");
        let v6addr = self.netlink.get_addresses_all(RtAddrFamily::Inet6);
println!("*** init 99");
    }
}
