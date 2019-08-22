//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Netlink abstraction.
//

use std::io;
use std::io::{Error, ErrorKind};
use std::str;
use std::str::FromStr;
use std::mem::{size_of, zeroed};
use std::rc::Rc;
use std::rc::Weak;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use libc::{self, c_void, c_int, c_uchar};
use log::debug;
use log::info;
use log::error;

use rtable::prefix::*;

use super::super::master::ZebraMaster;
use super::super::kernel::*;
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
        let rta_len = unsafe { (*rta).rta_len as usize };
        if rta_len >= size_of::<Rtattr>() && rta_len <= buf.len() {
            true
        } else {
            false
        }
    } else {
        false
    }
}

fn get_u32(p: &[u8]) -> u32 {
    unsafe { *(p as *const _ as *const u32) }
}

fn _get_u16(p: &[u8]) -> u16 {
    unsafe { *(p as *const _ as *const u16) }
}

fn _get_u8(p: &[u8]) -> u8 {
    unsafe { *(p as *const _ as *const u8) }
}

fn nlmsg_parse_attr<'a>(buf: &'a [u8]) -> AttrMap {
    let mut m = AttrMap::new();
    let mut b = &buf[..];

    while nlmsg_attr_ok(b) {
        let rta = b as *const _ as *const Rtattr;
        unsafe {
            let rta_len = (*rta).rta_len as usize;
            let rta_type = (*rta).rta_type as i32;
            let payload_len = rta_len - size_of::<Rtattr>();

            m.insert(rta_type, &b[size_of::<Rtattr>()..rta_len]);

            b = &b[nlmsg_align(rta_len)..];
        }
    }

    debug!("Attrs: {:?}", m.keys().collect::<Vec<&i32>>());

    m
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

    /// ZebraMaster.
    master: Weak<ZebraMaster>,

    /// Zebra Callbak functions.
    callbacks: KernelCallbacks,
}

impl Netlink {
    /// Constructor - open Netlink socket and bind.
    pub fn new(callbacks: KernelCallbacks) -> Result<Netlink, io::Error> {
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
            master: Default::default(),
            callbacks: callbacks,
        })
    }

    /// Set ZebraMaster.
    pub fn set_master(&mut self, master: Rc<ZebraMaster>) {
        self.master = Rc::downgrade(&master);
    }

    /// Send request message througn Netlink.
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

    /// Parse Netlink header and call parser to parse message payload.
    fn parse_info<T>(&self, parser: &Fn(&Netlink, &Nlmsghdr, &T, &AttrMap) -> bool) -> Result<(), io::Error> {
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
                libc::recvmsg(self.sock, &msg as *const _ as *mut libc::msghdr, 0)
            };
            if ret < 0 {
                info!("Error from recvmsg");
                // Check errno,
                // errno == EINTR
                //   break
                // errno == EWOULDBLOCK || errno == EAGAIN
                //   break
                // or something else
                // continue
                continue;
            } else if ret == 0 {
                info!("No data received");
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
                if p + size_of::<Nlmsghdr>() > recvlen {
                    error!("No enough space for Nlmsghdr in recv buffer.");
                    break;
                }

                let mut buf = &recvbuf[p..];
                let header = buf as *const _ as *const Nlmsghdr;
                let nlmsg_len = unsafe { (*header).nlmsg_len };
                let nlmsg_type = unsafe { (*header).nlmsg_type as i32 };
                buf = &buf[..nlmsg_len as usize];

                match nlmsg_type  {
                    libc::NLMSG_DONE => break 'outer,
                    libc::NLMSG_ERROR => {
                    },
                    _ => {
                    }
                }

                debug!("Nlmsg: type: {}, len: {}", nlmsg_type, nlmsg_len);

                if (nlmsg_len as usize) < nlmsg_attr::<T>() {
                    return Err(Error::new(ErrorKind::Other, "Insufficient Nlmsg length"))
                }

                let databuf = &buf[nlmsg_data()..];
                let data = databuf as *const _ as *const T;
                let attrbuf = &buf[nlmsg_attr::<T>()..];
                let map = nlmsg_parse_attr(attrbuf);
                let ret = unsafe { parser(self, &(*header), &(*data), &map) };
                // TODO check return value?

                p += nlmsg_len as usize;
            }
        }

        Ok(())
    }

    fn parse_interface(&self, h: &Nlmsghdr, ifi: &Ifinfomsg, attr: &AttrMap) -> bool {
        assert!(h.nlmsg_type == libc::RTM_NEWLINK);

        let ifindex = ifi.ifi_index;
        let mut hwaddr: [u8; 6] = match attr.get(&(libc::IFLA_ADDRESS as i32)) {
            Some(hwaddr) if hwaddr.len() == 6 => {
                [hwaddr[0], hwaddr[1], hwaddr[2], hwaddr[3], hwaddr[4], hwaddr[5]]
            },
            Some(hwaddr) => {
                error!("Invalid hwaddr length {}", hwaddr.len());
                [0, 0, 0, 0, 0, 0]
            },
            None => {
                [0, 0, 0, 0, 0, 0]
            }
        };

        let mtu = match attr.get(&(libc::IFLA_MTU as i32)) {
            Some(mtu) => get_u32(*mtu),
            None => 0u32,  // maybe set default?
        };
        let ifname = match attr.get(&(libc::IFLA_IFNAME as i32)) {
            Some(ifname) => {
                match str::from_utf8(ifname) {
                    Ok(ifname) => ifname,
                    Err(_) => "(Non-utf8)",
                }
            },
            None => "(Unknown)"
        };

        debug!("parse_interface() {} {} {} {:?} {}",
               ifi.ifi_index, ifname, ifi.ifi_type, hwaddr, mtu);

        // Call master to add Link.
        if let Some(master) = self.master.upgrade() {
            let link = Link::new(ifi.ifi_index, ifname, ifi.ifi_type as u16, hwaddr, mtu);
            (self.callbacks.add_link)(&master, link);

            true
        } else {
            error!("Callback failed");
            false
        }
    }

    fn parse_interface_address<T>(&self, h: &Nlmsghdr, ifa: &Ifaddrmsg, attr: &AttrMap) -> bool
    where T: AddressFamily + AddressLen + FromStr {
        assert!(h.nlmsg_type == libc::RTM_NEWADDR || h.nlmsg_type == libc::RTM_DELADDR);

        if ifa.ifa_family as i32 != T::address_family() {
            return false
        }

        let mut local = attr.get(&(libc::IFA_LOCAL as i32));
        let mut address = attr.get(&(libc::IFA_ADDRESS as i32));
        if let None = local {
            local = address;
        }
        if let None = address {
            address = local;
        }

        let broad = match address {
            Some(address) if address == local.unwrap() => {
                Some(address)
            },
            _ => {
                attr.get(&(libc::IFA_BROADCAST as i32))
            }
        };

        let index = ifa.ifa_index as i32;
        let prefix = Prefix::<T>::from_slice(address.unwrap(), ifa.ifa_prefixlen);
        let connected = Connected::<T>::new(prefix);

        if let Some(master) = self.master.upgrade() {
            match (h.nlmsg_type, ifa.ifa_family as i32) {
                (libc::RTM_NEWADDR, libc::AF_INET) =>
                    (self.callbacks.add_ipv4_address)(&master, index, connected),
                (libc::RTM_DELADDR, libc::AF_INET) =>
                    (self.callbacks.delete_ipv4_address)(&master, index, connected),
                (libc::RTM_NEWADDR, libc::AF_INET6) =>
                    (self.callbacks.add_ipv6_address)(&master, index, connected),
                (libc::RTM_DELADDR, libc::AF_INET6) =>
                    (self.callbacks.delete_ipv6_address)(&master, index, connected),
                (_, _) => assert!(false),
            };

            true
        } else {
            error!("Callback failed");
            true
        }
    }
}


impl LinkHandler for Netlink {
    /// Get all links from kernel.
    fn get_links_all(&self) -> Result<(), io::Error> {
        debug!("Get links all");

        if let Err(err) = self.send_request(libc::AF_PACKET, libc::RTM_GETLINK as i32) {
            error!("Send request: RTM_GETLINK");
            return Err(err)
        }

        if let Err(err) = self.parse_info(&Netlink::parse_interface) {
            error!("Parse info: RTM_GETLINK");
            return Err(err)
        }

        Ok(())
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
    /// Get all addresses per Address Family from kernel.
    fn get_addresses_all<T>(&self) -> Result<(), io::Error>
    where T: AddressFamily + AddressLen + FromStr {
        debug!("Get address all");

        if let Err(err) = self.send_request(T::address_family(), libc::RTM_GETADDR as i32) {
            error!("Send request: RTM_GETADDR");
            return Err(err)
        }

        if let Err(err) = self.parse_info(&Netlink::parse_interface_address::<T>) {
            error!("Parse info: RTM_GETADDR");
            return Err(err)
        }

        Ok(())
    }
}
