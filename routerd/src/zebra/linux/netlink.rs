//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//
use std::mem;
use std::str;
use std::cell::RefCell;

use libc::sockaddr_nl;

use neli::consts::*;
use neli::err::NlError;
use neli::nl::Nlmsghdr;
use neli::rtnl::*;
use neli::socket::*;

use super::super::link::*;
use super::super::address::*;
use super::super::route::*;

//use std::collections::HashMap;
//fn attr_vec2map<T: Eq, P: Copy>(v: Vec<Rtattr<T, P>>) -> HashMap<T, P> {
//    v.into_iter().map(|rta| (rta.rta_type, rta.rta_payload)).collect()
//}

fn attr_lookup_by_type<'a, T: Eq, P>(v: &'a Vec<Rtattr<T, P>>, t: T) -> Option<&'a P> {
    for a in v {
        if a.rta_type == t {
            return Some(&a.rta_payload)
        }
    }

    None
}

// Netlink.
pub struct Netlink {
    socket: RefCell<neli::socket::NlSocket>,
}

impl Netlink {
    // Constructor.
    pub fn new() -> Netlink {
        let socket = NlSocket::connect(NlFamily::Route, None, None, true).unwrap();

        // set buffersize
        // set non-blocking
        // pid ?

        Netlink {
            socket: RefCell::new(socket),
        }
    }

    fn send_request(&self, family: RtAddrFamily, nlmsg_type: u16) {
        let rtg = Rtgenmsg { rtgen_family: family };

        let nlh = {
            let len = None;
            let nlmsg_flags = vec![NlmF::Request, NlmF::Root, NlmF::Match];
            let nlmsg_seq = None;
            let nlmsg_pid = None;
            Nlmsghdr::new(len, nlmsg_type, nlmsg_flags, nlmsg_seq, nlmsg_pid, rtg)
        };

        if let Err(err) = self.socket.borrow_mut().send_nl(nlh) {
            println!("*** {:?}", err);
        }
    }

    fn parse_interface(rtm: Nlmsghdr<Rtm, Ifinfomsg<Ifla>>) -> Link {
        // rtm.nl_payload.ifi_family
        // rtm.nl_payload.ifi_type
        // rtm.nl_payload.ifi_index
        // rtm.nl_payload.ifi_flags
        // rtm.nl_payload.ifi_change?  not public
        // rtm.nl_payload.rtattrs

/*
        for rta in rtm.nl_payload.rtattrs {
            print!("rtattrs len={}, type={:?}, ", rta.rta_len, rta.rta_type);
            print!("{:?}", rta.rta_payload);
            println!("");
        }
*/
//        let map = attr_vec2map(rtm.nl_payload.rtattrs);
        let address = attr_lookup_by_type(&rtm.nl_payload.rtattrs, Ifla::Address).unwrap();
        let mtu = attr_lookup_by_type(&rtm.nl_payload.rtattrs, Ifla::Mtu).unwrap();
        let name = attr_lookup_by_type(&rtm.nl_payload.rtattrs, Ifla::Ifname).unwrap();

        let ifname = str::from_utf8(&name).unwrap();
        let hwaddr: [u8; 6] = [address[0], address[1], address[2], address[3], address[4], address[5]];
        let mtuv: u16 = (mtu[1] as u16) << 8 | mtu[0] as u16;

        Link::new(rtm.nl_payload.ifi_index, ifname, hwaddr, mtuv)
    }

    fn parse_info<P: neli::Nl, D>(&self, parser: &Fn(Nlmsghdr<Rtm, P>) -> D) -> Vec<D> {
        let mut v = Vec::new();

        while let Ok(nl) = self.socket.borrow_mut().recv_nl::<u16, P>(None) {
            match Nlmsg::from(nl.nl_type) {
                Nlmsg::Done => {
                    // We want to log probably.
                    println!("Done");
                    break;
                },
                Nlmsg::Error => {
                    // We want to log probably.
                    println!("Error");
                    break;
                }
                _ => {
                    let rtm = Nlmsghdr {
                        nl_len: nl.nl_len,
                        nl_type: Rtm::from(nl.nl_type),
                        nl_flags: nl.nl_flags,
                        nl_seq: nl.nl_seq,
                        nl_pid: nl.nl_pid,
                        nl_payload: nl.nl_payload,
                    };

                    v.push(parser(rtm));
                }
            }
        }

        v
    }

//    pub fn init(&mut self) {
//        self.send_request(RtAddrFamily::Packet, Rtm::Getlink.into());
//        self.parse_info(&Netlink::parse_interface);
//    }
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
