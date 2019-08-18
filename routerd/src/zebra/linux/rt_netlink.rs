//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//
use std::mem;
use std::str;
use std::cell::RefCell;
use std::net::IpAddr;

use libc::sockaddr_nl;

//use neli::consts::*;
//use neli::err::NlError;
//use neli::nl::Nlmsghdr;
//use neli::rtnl::*;
//use neli::socket::*;

use super::netlink;

use super::super::link::*;
use super::super::address::*;
use super::super::route::*;

// Netlink.
pub struct RtNetlink {
    //
    netlink: RefCell<netlink::NetlinkSocket>,

    //
}

impl RtNetlink {
    // Constructor.
    pub fn new() -> RtNetlink {
//        let mut socket = NlSocket::connect2(NlFamily::Route, Some(group), true).unwrap();

//        socket.nonblock().unwrap();

        // set buffersize
        // set non-blocking
        // pid ?

        RtNetlink {
            socket: RefCell::new(socket),
        }
    }

    fn send_request(&self, family: RtAddrFamily, nlmsg_type: u16) {
        let rtg = Rtgenmsg { rtgen_family: family };

        let nlh = {
            let len = None;
            let nlmsg_flags = vec![NlmF::Request, NlmF::Root, NlmF::Match];
            let nlmsg_seq = None;
            Nlmsghdr::new(len, nlmsg_type, nlmsg_flags, nlmsg_seq, None/*, Some(rtg)*/)
        };
        // should send payload

        if let Err(err) = self.socket.borrow_mut().send_nl(nlh) {
            println!("Error: socket.send_nl() {:?}", err);
        }
    }

    fn parse_info<D>(&self, parser: &Fn(Nlmsghdr<Rtm>) -> D) -> Vec<D> {
        let mut v = Vec::new();
        println!("** parse_info 00");

        while let Ok((nl, payload)) = self.socket.borrow_mut().recv_nl::<Rtm>(None) {
        println!("** parse_info 10");
            match nl.nl_type {
                Rtm::Done => {
                    // We want to log probably.
                    println!("Done");
                    break;
                },
                Rtm::Error => {
                    // We want to log probably.
                    println!("Error");
                    break;
                }
                _ => {
                    v.push(parser(nl));
                }
            }
        }
        self.socket.borrow_mut().reset_buffer();

        println!("** parse_info 99");
        v
    }

    fn parse_interface(rtm: Nlmsghdr<Rtm/*, Ifinfomsg<Ifla>*/>) -> Link {
        // rtm.nl_payload.ifi_family
        // rtm.nl_payload.ifi_type
        // rtm.nl_payload.ifi_index
        // rtm.nl_payload.ifi_flags
        // rtm.nl_payload.ifi_change?  not public
        // rtm.nl_payload.rtattrs

        let mut hwaddr: [u8; 6] = [0, 0, 0, 0, 0, 0];
        let mut mtu = None;
        let mut ifname = None;

/*
        for attr in &rtm.nl_payload.as_ref().unwrap().rtattrs {
            fn to_u32(b: &[u8]) -> u32 {
                b[0] as u32 | (b[1] as u32) << 8 | (b[2] as u32) << 16 | (b[3] as u32) << 24
            }

            match attr.rta_type {
                Ifla::Address => {
                    hwaddr[..6].clone_from_slice(&attr.rta_payload);
                },
                Ifla::Mtu => {
                    mtu = Some(to_u32(&attr.rta_payload));
                },
                Ifla::Ifname => {
                    ifname = Some(str::from_utf8(&attr.rta_payload).unwrap());
                },
                _ => {
                    //
                }
            }
        }
*/

//        Link::new(rtm.nl_payload.as_ref().unwrap().ifi_index, ifname.unwrap(), hwaddr, mtu.unwrap())

        // somehow we get payload 

        Link::new(0, ifname.unwrap(), hwaddr, mtu.unwrap())
    }

    fn parse_interface_address(rtm: Nlmsghdr<Rtm/*, Ifaddrmsg<Ifa>*/>) -> Connected {
//        println!("*** prefixlen {}", rtm.nl_payload.as_ref().unwrap().ifa_prefixlen);

/*
        let mut local = None;
        let mut address = None;
        let mut broadcast = None;
*/

/*
        for attr in &rtm.nl_payload.as_ref().unwrap().rtattrs {
            fn to_addr(b: &[u8]) -> Option<IpAddr> {
                use std::convert::TryFrom;
                if let Ok(tup) = <&[u8; 4]>::try_from(b) {
                    Some(IpAddr::from(*tup))
                } else if let Ok(tup) = <&[u8; 16]>::try_from(b) {
                    Some(IpAddr::from(*tup))
                } else {
                    None
                }
            }

            match attr.rta_type {
                Ifa::Local => {
                    local = to_addr(&attr.rta_payload);
                },
                Ifa::Address => {
                    address = to_addr(&attr.rta_payload);
                },
                Ifa::Broadcast => {
                    broadcast = to_addr(&attr.rta_payload);
                },
                _ => {
                }
            }
        }

        if let Some(local) = local {
            println!("*** local {}", local);
        }
        if let Some(address) = address {
            println!("*** address {}", address);
        }
        if let Some(broadcast) = broadcast {
            println!("*** broadcast {}", broadcast);
        }
*/

        Connected::new()
    }
}

impl LinkHandler for Netlink {
    // Get all links from kernel.
    fn get_links_all(&self) -> Vec<Link> {
        self.send_request(RtAddrFamily::Packet, Rtm::Getlink.into());
        self.parse_info(&Netlink::parse_interface)
    }

    // Add link from zebra
    //fn add_link(&self) -> ?

    // Get link information.
    fn get_link(&self, index: i32) -> Option<Link> {
        None
    }

    // Set MTU.
    fn set_mtu(&self, mtu: u16) -> bool {
        true
    }

    // Set link up.
    fn set_link_up(&self) -> bool {
        true
    }

    // Set link down.
    fn set_link_down(&self) -> bool {
        true
    }

    // Set callback for link stat change.
//    fn set_link_change_callback(&self, &Fn());
}

impl AddressHandler for Netlink {
    // Get all addresses from kernel
    fn get_addresses_all(&self, family: RtAddrFamily) -> Vec<Connected> {
        self.send_request(family, Rtm::Getaddr.into());
        self.parse_info(&Netlink::parse_interface_address)
    }
}
