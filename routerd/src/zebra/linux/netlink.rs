//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//

use std::io;
use std::str;
use std::mem::{size_of, zeroed};
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use libc::{self, c_void, c_int, c_uchar};
//use std::net::IpAddr;

use super::super::link::*;
use super::super::address::*;
use super::super::route::*;

const RTMGRP_LINK: libc::c_int = 1;
const RTMGRP_IPV4_IFADDR: libc::c_int = 0x10;
const RTMGRP_IPV4_ROUTE: libc::c_int = 0x40;
const RTMGRP_IPV6_IFADDR: libc::c_int = 0x100;
const RTMGRP_IPV6_ROUTE: libc::c_int = 0x400;

const NETLINK_RECV_BUFSIZ: usize = 4096;

const NLMSG_ALIGNTO: usize = 4usize;

fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}

fn nlmsg_data() -> usize {
    size_of::<libc::nlmsghdr>()
}

fn nlmsg_attr<T>() -> usize {
    nlmsg_data() + nlmsg_align(size_of::<T>())
}

fn nlmsg_attr_ok(buf: &[u8]) -> bool {
    // Ensure the size of buffer has enough space for Rtattr.
    if buf.len() >= size_of::<Rtattr>() {
        let rta = buf as *const _ as *const Rtattr;
        unsafe {
            if (*rta).rta_len as usize >= size_of::<Rtattr>() && (*rta).rta_len as usize <= buf.len() {
                true
            } else {
                false
            }
        }
    } else {
        false
    }
}

fn nlmsg_parse_attr<'a>(buf: &'a [u8]) -> AttrMap {
    let mut map = AttrMap::new();
    let mut b = &buf[..];

    while nlmsg_attr_ok(b) {
        let rta = b as *const _ as *const Rtattr;
        unsafe {
            let rta_len = (*rta).rta_len as usize;
            let rta_type = (*rta).rta_type as i32;
            let payload_len = rta_len - size_of::<Rtattr>();

println!("*** parse_attr type {}, len {}", rta_type, rta_len);

            map.insert(rta_type, &b[size_of::<Rtattr>()..rta_len]);

            b = &b[nlmsg_align(rta_len)..];
        }
    }
println!(">>>>>>>>>>> parse_attr return map");

    map
}

/// Typedefs.
type Nlmsghdr = libc::nlmsghdr;
type AttrMap<'a> = HashMap<c_int, &'a [u8]>;

/// struct rtattr
#[repr(C)]
struct Rtattr {
    rta_len: u16,
    rta_type: u16,
}

/// struct rtgenmsg from rtnetlink.h.
#[repr(C)]
struct Rtgenmsg {
    rtgen_family: libc::c_uchar
}

/// struct ifinfomsg from rtnetlink.h.
#[repr(C)]
struct Ifinfomsg {
    ifi_family: u8,
    _ifi_pad: u8,
    ifi_type: u16,
    ifi_index: i32,
    ifi_flags: u32,
    ifi_change: u32,
}

/// struct ifaddrmsg from if_addr.h.
#[repr(C)]
struct Ifaddrmsg {
    ifa_family: u8,
    ifa_prefixlen: u8,
    ifa_flags: u8,
    ifa_scope: u8,
    ifa_index: u32,
}

struct Buffer {
    p: [u8; NETLINK_RECV_BUFSIZ],
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            p: [0; NETLINK_RECV_BUFSIZ],
        }
    }
}

/// Netlink Socket handler.
pub struct Netlink {
    /// File descriptor for Netlink socket.
    sock: c_int,
    
    /// PID associated with this Netlink socket.
    pid: u32,

    /// Sequence number of messsage.
    seq: Cell<u32>,

    /// Receive buffer.
    buf: RefCell<Buffer>,
}

impl Netlink {
    /// Constructor - open Netlink socket and bind.
    pub fn new() -> Result<Netlink, io::Error> {
        let sock = unsafe {
            libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, libc::NETLINK_ROUTE)
        };
        if sock < 0 {
            return Err(io::Error::last_os_error());
        };

        let mut snl = unsafe { zeroed::<libc::sockaddr_nl>() };
        snl.nl_family = libc::AF_NETLINK as u16;
        snl.nl_groups = RTMGRP_LINK as u32 |
                        RTMGRP_IPV4_IFADDR as u32 | RTMGRP_IPV4_ROUTE as u32 |
                        RTMGRP_IPV6_IFADDR as u32 | RTMGRP_IPV6_ROUTE as u32;
        let mut socklen: libc::socklen_t = size_of::<libc::sockaddr_nl>() as u32;
        let ret = unsafe {
            libc::bind(
                sock,
                &snl as *const _ as *const libc::sockaddr,
                socklen,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let ret = unsafe {
            libc::getsockname(
                sock,
                &mut snl as *const _ as *mut libc::sockaddr,
                &mut socklen)
        };
        if ret < 0 || socklen != size_of::<libc::sockaddr_nl>() as u32 {
            return Err(io::Error::last_os_error());
        }

        // TODO: set socket non-blocking.

        Ok(Netlink {
            sock,
            pid: snl.nl_pid,
            seq: Cell::new(0u32),
            buf: RefCell::new(Buffer::new()),
        })
    }

    ///
    fn send_request(&self, family: libc::c_int, nlmsg_type: libc::c_int) -> Result<(), io::Error> {
        struct Request {
            nlmsghdr: Nlmsghdr,
            rtgenmsg: Rtgenmsg,
        }

        let seq = self.seq.get() + 1;
        self.seq.set(seq);

        let mut snl = unsafe { zeroed::<libc::sockaddr_nl>() };
        snl.nl_family = libc::AF_NETLINK as u16;
        
        let mut req = unsafe { zeroed::<Request>() };
        req.nlmsghdr.nlmsg_len = size_of::<Request>() as u32;
        req.nlmsghdr.nlmsg_type = nlmsg_type as u16;
        req.nlmsghdr.nlmsg_flags = libc::NLM_F_ROOT as u16 |
                                   libc::NLM_F_MATCH as u16 |
                                   libc::NLM_F_REQUEST as u16;
        req.nlmsghdr.nlmsg_pid = self.pid;
        req.nlmsghdr.nlmsg_seq = seq;
        req.rtgenmsg.rtgen_family = family as u8;

        let ret = unsafe {
            libc::sendto(self.sock,
                         &req as *const _ as *const libc::c_void,
                         size_of::<Request>(), 0,
                         &snl as *const _ as *const libc::sockaddr,
                         size_of::<libc::sockaddr_nl>() as u32)
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    fn parse_info<T, D>(&self, parser: &Fn(&Nlmsghdr, &T, &AttrMap) -> D) -> Vec<D> {
        let mut v = Vec::new();

        'outer: loop {
            let mut buffer = self.buf.borrow_mut();

            let mut iov = unsafe { zeroed::<libc::iovec>() };
            iov.iov_base = &mut buffer.p as *const _ as *mut libc::c_void;
            iov.iov_len = NETLINK_RECV_BUFSIZ;
            let mut snl = unsafe { zeroed::<libc::sockaddr_nl>() };
            let mut msg = unsafe { zeroed::<libc::msghdr>() };
            msg.msg_name = &mut snl as *const _ as *mut libc::c_void;
            msg.msg_namelen = size_of::<libc::sockaddr_nl>() as u32;
            msg.msg_iov = &mut iov as *const _ as *mut libc::iovec;
            msg.msg_iovlen = 1;

            let ret = unsafe {
                libc::recvmsg(self.sock,
                              &msg as *const _ as *mut libc::msghdr,
                              0)
            };
            if ret < 0 {
                println!("*** parse_info Error");
                // Check errno,
                // errno == EINTR
                //   break
                // errno == EWOULDBLOCK || errno == EAGAIN
                //   break
                // or something else
                // continue
                continue;
            } else if ret == 0 {
                println!("*** parse_info no Data");
                break 'outer;
            }

            if msg.msg_namelen != size_of::<libc::sockaddr_nl>() as u32 {
                // sender address length error
                break 'outer;
            }

            let recvlen = ret as usize;
            let mut recvbuf = &buffer.p[..recvlen];
            let mut p = 0;
            while p < recvlen {
                unsafe {
                    let mut buf = &recvbuf[p..];
                    let header = buf as *const _ as *const Nlmsghdr;
                    let nlmsg_len = (*header).nlmsg_len;
                    println!("*** nlmsg_len {}", (*header).nlmsg_len);
                    println!("*** nlmsg_type {}", (*header).nlmsg_type);
                    buf = &buf[..nlmsg_len as usize];

                    match (*header).nlmsg_type as i32 {
                        libc::NLMSG_DONE => break 'outer,
                        libc::NLMSG_ERROR => {
                        },
                        _ => {

                        }
                    }

                    // TODO: debug message
                    let databuf = &buf[nlmsg_data()..];
                    let data = databuf as *const _ as *const T;
                    let attrbuf = &buf[nlmsg_attr::<T>()..];
                    let map = nlmsg_parse_attr(attrbuf);
                    let ret = parser(&(*header), &(*data), &map);

                    p += nlmsg_len as usize;
                }
            }
        }

        v
    }

    fn parse_interface(h: &Nlmsghdr, ifi: &Ifinfomsg, attr: &AttrMap) -> Link {
        let mut hwaddr: [u8; 6] = [0, 0, 0, 0, 0, 0];
//        let mut mtu = None;
        let ifname = match attr.get(&(libc::IFLA_IFNAME as i32)) {
            Some(ifname) => {
                match str::from_utf8(ifname) {
                    Ok(ifname) => ifname,
                    Err(_) => "(Non-utf8)",
                }
            },
            None => "(Unknown)"
        };
println!("*** {:?}", ifname);


        Link::new(0, /*ifname*/"hoge", hwaddr, 1500)
    }

    fn parse_interface_address(h: &Nlmsghdr, ifa: &Ifaddrmsg, attr: &AttrMap) -> Connected {
        Connected::new()
    }
}


impl LinkHandler for Netlink {
    // Get all links from kernel.
    fn get_links_all(&self) -> Vec<Link> {
println!("*** get_links_all 00");
        match self.send_request(libc::AF_PACKET, libc::RTM_GETLINK as i32) {
            Ok(_) => { println!("*** OK"); },
            Err(_) =>  { println!("*** ERR"); },
        };
println!("*** get_links_all 10");
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
    fn get_addresses_all(&self, family: libc::c_int) -> Vec<Connected> {
        self.send_request(family, libc::RTM_GETADDR as i32);
        self.parse_info(&Netlink::parse_interface_address)
    }
}
