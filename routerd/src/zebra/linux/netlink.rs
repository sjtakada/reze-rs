//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//

use std::io;
use std::mem::{size_of, zeroed};
use libc::{self, c_int};
use std::cell::Cell;

//use std::str;
//use std::net::IpAddr;

/// Netlink Socket handler.
pub struct NetlinkSocket {
    /// File descriptor for Netlink socket.
    sock: c_int,
    
    /// PID associated with this Netlink socket.
    pid: u32,

    /// Sequence number of messsage.
    seq: Cell<u32>,

    /// struct sockaddr_nl snl,
}

impl NetlinkSocket {
    pub fn new(protocol: c_int) -> Result<NetlinkSocket, io::Error> {
        let sock = match unsafe {
            libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, protocol)
        } {
            sock if sock >= 0 => sock,
            _ => Err(io::Error::last_os_error()),
        }?;

        let mut snl = unsafe { zeroed::<libc::sockaddr_nl>() };
        snl.nl_family = libc::AF_NETLINK;
        snl.groups = (libc::RTMGRP_LINK |
                      libc::RTMGRP_IPV4_ADDR | libc::RTMGRP_IPV4_ROUTE |
                      libc::RTMGRP_IPV6_ADDR | libc::RTMGRP_IPV6_ROUTE);
        let mut socklen: libc::socklen_t = size_of::<libc::sockaddr_nl><() as u32;
        match unsafe {
            libc::bind(
                self.fd,
                &nladdr as *const _ as *const libc::sockaddr,
                socklen,
            )
        } {
            i if i >= 0 => (),
            _ => return Err(io::Error::last_os_error()),
        };

        match unsafe {
            libc::getsockname(
                self.fd,
                &mut snl as *const _ as *mut libc::sockaddr,
                &mut socklen)
        } {
            i if i == 0 && socklen == size_of::<libc::sockaddr_nl>() as u32 => (),
            _ => return Err(io::Error::last_os_error()),
        };

        Ok(Netlink {
            sock,
            snl.nl_pid,
            seq: Cell(0u32),
        })
    }
}
