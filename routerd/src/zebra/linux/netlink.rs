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
use std::ptr::copy;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use libc::{self, c_int/*, c_void, c_uchar*/};
use log::debug;
use log::info;
use log::error;

use rtable::prefix::*;

use super::rtnetlink::*;
use super::super::master::ZebraMaster;
use super::super::kernel::*;
use super::super::link::*;
use super::super::address::*;
use super::super::rib::*;
//use super::super::route::*;

const RTMGRP_LINK: libc::c_int = 1;
const RTMGRP_IPV4_IFADDR: libc::c_int = 0x10;
const RTMGRP_IPV4_ROUTE: libc::c_int = 0x40;
const RTMGRP_IPV6_IFADDR: libc::c_int = 0x100;
const RTMGRP_IPV6_ROUTE: libc::c_int = 0x400;

const RTPROT_ZEBRA: libc::c_int = 11;

const NETLINK_RECV_BUFSIZ: usize = 4096;

const NLMSG_ALIGNTO: usize = 4usize;

fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO - 1) & !(NLMSG_ALIGNTO - 1)
}

fn nlmsg_hdrlen() -> usize {
    nlmsg_align(size_of::<libc::nlmsghdr>())
}

fn nlmsg_length(len: usize) -> usize {
    nlmsg_hdrlen() + len
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

fn addattr_l(h: &mut Nlmsghdr, maxlen: usize, rta_type: libc::c_int, src: &[u8], alen: usize) -> bool {
    let len = rta_length(alen);
    let ptr = h as *const _ as *const usize;

    if nlmsg_align(h.nlmsg_len as usize) + len > maxlen {
        false
    } else {
        unsafe {
            let src_ptr = src as *const _ as *mut libc::c_void;

            let rta = ptr.offset(nlmsg_align(h.nlmsg_len as usize) as isize) as *mut Rtattr;
            let rta_ptr = rta as *const _ as *mut libc::c_void;
            let dst_ptr = rta_ptr.offset(size_of::<Rtattr>() as isize);

            (*rta).rta_len = len as u16;
            (*rta).rta_type = rta_type as u16;
            
            copy(src_ptr, dst_ptr, alen);

            h.nlmsg_len = (nlmsg_align(h.nlmsg_len as usize) + len) as u32;
        }

        true
    }
}

fn addattr32(h: &mut Nlmsghdr, maxlen: usize, rta_type: libc::c_int, src: u32) -> bool {
    let len = rta_length(size_of::<u32>());
    let ptr = h as *const _ as *const usize;

    if nlmsg_align(h.nlmsg_len as usize) + len > maxlen {
        false
    } else {
        unsafe {
            let src_ptr = &src as *const _ as *mut libc::c_void;

            let rta = ptr.offset(nlmsg_align(h.nlmsg_len as usize) as isize) as *mut Rtattr;
            let rta_ptr = rta as *const _ as *mut libc::c_void;
            let dst_ptr = rta_ptr.offset(size_of::<Rtattr>() as isize);

            (*rta).rta_len = len as u16;
            (*rta).rta_type = rta_type as u16;
            
            copy(src_ptr, dst_ptr, size_of::<u32>());

            h.nlmsg_len = (nlmsg_align(h.nlmsg_len as usize) + len) as u32;
        }

        true
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
            let _payload_len = rta_len - size_of::<Rtattr>();

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

/// struct rtmsg from rtnetlink.h.
#[repr(C)]
struct Rtmsg {
    rtm_family: u8,
    rtm_dst_len: u8,
    rtm_src_len: u8,
    rtm_tos: u8,

    rtm_table: u8,
    rtm_protocol: u8,
    rtm_scope: u8,
    rtm_type: u8,

    rtm_flags: u32,
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

/// Dummy placeholder for netlink_talk
#[repr(C)]
struct NlDummy {
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

    /// Install route to kernel.
    pub fn install<P: Prefixable>(&self, prefix: &P, rib: &Rib<P>) {
        self.route_msg(libc::RTM_NEWROUTE as i32, prefix);
    }

    /// Build route message.
    fn route_msg<P: Prefixable>(&self, /*h: &mut Nlmsghdr,*/ cmd: libc::c_int, prefix: &P) -> Result<(), io::Error> {
        println!("*** route_msg");

        let family = libc::AF_INET;
        let bytelen = if family == libc::AF_INET { 4 } else { 16 };

        #[repr(C)]
        struct Request {
            nlmsghdr: Nlmsghdr,
            rtmsg: Rtmsg,
            buf: [u8; 4096],	// XXX
        }

        let mut req = unsafe { zeroed::<Request>() };

        req.nlmsghdr.nlmsg_len = nlmsg_length(size_of::<Rtmsg>()) as u32;
        req.nlmsghdr.nlmsg_flags = libc::NLM_F_CREATE as u16 |
                                   libc::NLM_F_REPLACE as u16 |
                                   libc::NLM_F_REQUEST as u16;
        req.nlmsghdr.nlmsg_type = cmd as u16;
        req.rtmsg.rtm_family = family as u8;
        req.rtmsg.rtm_table = libc::RT_TABLE_MAIN as u8;
        req.rtmsg.rtm_dst_len = prefix.len();
        req.rtmsg.rtm_protocol = RTPROT_ZEBRA as u8;
        req.rtmsg.rtm_scope = libc::RT_SCOPE_LINK as u8;
        req.rtmsg.rtm_type = libc::RTN_UNICAST as u8;

        if cmd == libc::RTM_NEWROUTE as i32 {
            req.rtmsg.rtm_type = libc::RTN_UNICAST;
        }

        // Destination address.
        addattr_l(&mut req.nlmsghdr, size_of::<Request>(), libc::RTA_DST as i32, prefix.octets(), bytelen);

        // Metric.
        addattr32(&mut req.nlmsghdr, size_of::<Request>(), libc::RTA_PRIORITY as i32, 1);

        // Nexthops.
        let nexthop: [u8; 4] = [172, 16, 0, 100];
        addattr_l(&mut req.nlmsghdr, size_of::<Request>(), libc::RTA_GATEWAY as i32, &nexthop, 4);

println!("*** nlmsg_len {}", req.nlmsghdr.nlmsg_len);

        unsafe {
            let x = &req as *const _ as *mut libc::c_char;
            let mut i: usize = 0;
            while i < req.nlmsghdr.nlmsg_len as usize {
                println!("*** {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                         *(x.add(i + 0)),
                         *(x.add(i + 1)),
                         *(x.add(i + 2)),
                         *(x.add(i + 3)),
                         *(x.add(i + 4)),
                         *(x.add(i + 5)),
                         *(x.add(i + 6)),
                         *(x.add(i + 7)));
                i += 8;
            }
        }

        // Send command message through Netlink socket.
        self.send_command(&mut req.nlmsghdr)
    }

    /// Send a command through Netlink.
    fn send_command(&self, mut h: &mut Nlmsghdr) -> Result<(), io::Error> {
        let mut snl = unsafe { zeroed::<libc::sockaddr_nl>() };
        snl.nl_family = libc::AF_NETLINK as u16;

        let mut iov = unsafe { zeroed::<libc::iovec>() };
        iov.iov_base = &mut h as *const _ as *mut libc::c_void;
        iov.iov_len = h.nlmsg_len as usize;

        let mut msg = unsafe { zeroed::<libc::msghdr>() };
        msg.msg_name = &mut snl as *const _ as *mut libc::c_void;
        msg.msg_namelen = size_of::<libc::sockaddr_nl>() as u32;
        msg.msg_iov = &mut iov as *const _ as *mut libc::iovec;
        msg.msg_iovlen = 1;

        let seq = self.seq.get() + 1;
        self.seq.set(seq);

        h.nlmsg_seq = seq;
        h.nlmsg_flags |= libc::NLM_F_ACK as u16;

println!("*** send 1");

        let ret = unsafe {
            libc::sendmsg(self.sock,
                          &msg as *const _ as *const libc::msghdr, 0)
        };

println!("*** send 2 {}", ret);
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let a = self.parse_info(&Netlink::parse_dummy);

println!("*** send 3");

        a
    }

    /// Send a request message through Netlink.
    /// Expect to receive response.
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
    fn parse_info<T>(&self, parser: &dyn Fn(&Netlink, &Nlmsghdr, &T, &AttrMap) -> bool) -> Result<(), io::Error> {
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
            let recvbuf = &buffer.p[..recvlen];
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
                let _ret = unsafe { parser(self, &(*header), &(*data), &map) };
                // TODO check return value?

                p += nlmsg_len as usize;
            }
        }

        Ok(())
    }

    fn parse_dummy(&self, h: &Nlmsghdr, ifi: &NlDummy, attr: &AttrMap) -> bool {
        debug!("Nlmsg type {}", h.nlmsg_type);

        true
    }

    fn parse_interface(&self, h: &Nlmsghdr, ifi: &Ifinfomsg, attr: &AttrMap) -> bool {
        assert!(h.nlmsg_type == libc::RTM_NEWLINK);

        let ifindex = ifi.ifi_index;
        let hwaddr: [u8; 6] = match attr.get(&(libc::IFLA_ADDRESS as i32)) {
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
               ifindex, ifname, ifi.ifi_type, hwaddr, mtu);

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

        let _broad = match address {
            Some(address) if address == local.unwrap() => {
                Some(address)
            },
            _ => {
                attr.get(&(libc::IFA_BROADCAST as i32))
            }
        };

        let index = ifa.ifa_index as i32;

        if let Some(master) = self.master.upgrade() {
            match ifa.ifa_family as i32 {
                libc::AF_INET => {
                    let prefix = Prefix::<Ipv4Addr>::from_slice(address.unwrap(), ifa.ifa_prefixlen);
                    let connected = Connected::<Ipv4Addr>::new(prefix);

                    match h.nlmsg_type {
                        libc::RTM_NEWADDR =>
                            (self.callbacks.add_ipv4_address)(&master, index, connected),
                        libc::RTM_DELADDR =>
                            (self.callbacks.delete_ipv4_address)(&master, index, connected),
                        _ => assert!(false),
                    }
                },
                libc::AF_INET6 => {
                    let prefix = Prefix::<Ipv6Addr>::from_slice(address.unwrap(), ifa.ifa_prefixlen);
                    let connected = Connected::<Ipv6Addr>::new(prefix);

                    match h.nlmsg_type {
                        libc::RTM_NEWADDR =>
                            (self.callbacks.add_ipv6_address)(&master, index, connected),
                        libc::RTM_DELADDR =>
                            (self.callbacks.delete_ipv6_address)(&master, index, connected),
                        _ => assert!(false),
                    }
                },
                _ => assert!(false),
            }

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
    fn set_mtu(&self, _mtu: u16) -> bool {
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
