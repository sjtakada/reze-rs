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

// Netlink.
pub struct Netlink {
    //
    socket: RefCell<neli::socket::NlSocket>,

    //
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
            println!("Error: socket.send_nl() {:?}", err);
        }
    }

    fn parse_interface(rtm: Nlmsghdr<Rtm, Ifinfomsg<Ifla>>) -> Link {
        // rtm.nl_payload.ifi_family
        // rtm.nl_payload.ifi_type
        // rtm.nl_payload.ifi_index
        // rtm.nl_payload.ifi_flags
        // rtm.nl_payload.ifi_change?  not public
        // rtm.nl_payload.rtattrs

        let mut hwaddr: [u8; 6] = [0, 0, 0, 0, 0, 0];
        let mut mtu = None;
        let mut ifname = None;

        for attr in &rtm.nl_payload.rtattrs {
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

        Link::new(rtm.nl_payload.ifi_index, ifname.unwrap(), hwaddr, mtu.unwrap())
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
