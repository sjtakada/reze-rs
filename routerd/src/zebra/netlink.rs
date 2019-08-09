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
struct Netlink {
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

    pub fn send_request(&mut self, family: u8, nlmsg_type: u16) {
        let rtg = Rtgenmsg { rtgen_family: family };

        let nlh = {
            let len = None;
            let nlmsg_flags = vec![NlmF::Request, NlmF::Root, NlmF::Match];
            let nlmsg_seq = None;
            let nlmsg_pid = None;
            Nlmsghdr::new(len, nlmsg_type, nlmsg_flags, nlmsg_seq, nlmsg_pid, rtg)
        };

//        let mut snl = unsafe { mem::zeroed::<libc::sockaddr_nl>() };
//        snl.nl_family = libc::c_int::from(AddrFamily::Netlink) as u16;

        self.socket.send_nl(nlh).unwrap();
    }
}

