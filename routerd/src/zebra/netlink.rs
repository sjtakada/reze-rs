//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//
use std::mem;
use std::cell::Cell;

use libc::sockaddr_nl;

use neli::consts::*;
use neli::err::NlError;
use neli::nl::Nlmsghdr;
use neli::rtnl::*;
use neli::socket::*;

// Netlink.
pub struct Netlink {
    socket: neli::socket::NlSocket,
}

impl Netlink {
    // Constructor.
    pub fn new() -> Netlink {
        let mut socket = NlSocket::connect(NlFamily::Route, None, None, true).unwrap();

        // set buffersize
        // set non-blocking
        // pid ?

        Netlink {
            socket: socket,
        }
    }

    pub fn send_request(&mut self, family: RtAddrFamily, nlmsg_type: u16) {
        let rtg = Rtgenmsg { rtgen_family: family };

        let nlh = {
            let len = None;
            let nlmsg_flags = vec![NlmF::Request, NlmF::Root, NlmF::Match];
            let nlmsg_seq = None;
            let nlmsg_pid = None;
            Nlmsghdr::new(len, nlmsg_type, nlmsg_flags, nlmsg_seq, nlmsg_pid, rtg)
        };

        if let Err(err) = self.socket.send_nl(nlh) {
            println!("*** {:?}", err);
        }
    }

    pub fn parse_interface(&self, rtm: Nlmsghdr<Rtm, Ifinfomsg<Ifla>>) {
        // rtm.nl_payload.ifi_family
        // rtm.nl_payload.ifi_type
        // rtm.nl_payload.ifi_index
        // rtm.nl_payload.ifi_flags
        // rtm.nl_payload.ifi_change?  not public
        // rtm.nl_payload.rtattrs
        println!("ifi_family {:?}", rtm.nl_payload.ifi_family);
        println!("ifi_type   {:?}", rtm.nl_payload.ifi_type);
        println!("ifi_index  {:?}", rtm.nl_payload.ifi_index);
//        println!("ifi_flags  {}", rtm.nl_payload.ifi_flags);

        for rta in rtm.nl_payload.rtattrs {
            print!("rtattrs len={}, type={:?}, ", rta.rta_len, rta.rta_type);
            print!("{:?}", rta.rta_payload);
            println!("");
        }
    }

    pub fn parse_info(&mut self) {
        println!("*** parse_info");
        while let Ok(nl) = self.socket.recv_nl::<u16, Ifinfomsg<Ifla>>(None) {
            match Nlmsg::from(nl.nl_type) {
                Nlmsg::Done => { println!("OK"); return; }
                Nlmsg::Error => { println!("Err"); }
                _ => {
                    let rtm = Nlmsghdr {
                        nl_len: nl.nl_len,
                        nl_type: Rtm::from(nl.nl_type),
                        nl_flags: nl.nl_flags,
                        nl_seq: nl.nl_seq,
                        nl_pid: nl.nl_pid,
                        nl_payload: nl.nl_payload,
                    };

                    self.parse_interface(rtm);
                }
            }
        }
    }

    pub fn init(&mut self) {
        self.send_request(RtAddrFamily::Packet, Rtm::Getlink.into());
        self.parse_info();
    }
}

