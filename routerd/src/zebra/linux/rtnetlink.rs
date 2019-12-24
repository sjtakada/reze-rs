//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Rtattributes abstraction.
//

/*
#define RTA_ALIGNTO     4U
#define RTA_ALIGN(len) ( ((len)+RTA_ALIGNTO-1) & ~(RTA_ALIGNTO-1) )
#define RTA_OK(rta,len) ((len) >= (int)sizeof(struct rtattr) && \
                         (rta)->rta_len >= sizeof(struct rtattr) && \
                         (rta)->rta_len <= (len))
#define RTA_NEXT(rta,attrlen)   ((attrlen) -= RTA_ALIGN((rta)->rta_len), \
                                 (struct rtattr*)(((char*)(rta)) + RTA_ALIGN((rta)->rta_len)))
#define RTA_LENGTH(len) (RTA_ALIGN(sizeof(struct rtattr)) + (len))
#define RTA_SPACE(len)  RTA_ALIGN(RTA_LENGTH(len))
#define RTA_DATA(rta)   ((void*)(((char*)(rta)) + RTA_LENGTH(0)))
#define RTA_PAYLOAD(rta) ((int)((rta)->rta_len) - RTA_LENGTH(0))

*/

use std::mem::size_of;
use std::ptr::copy;

use super::netlink::*;


const RTA_ALIGNTO: usize = 4usize;

pub fn rta_align(len: usize) -> usize {
    (len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1)
}

pub fn rta_length(len: usize) -> usize {
    rta_align(size_of::<Rtattr>()) + len
}

pub fn rta_data() -> usize {
    size_of::<Rtattr>()
}

/// struct rtattr
#[repr(C)]
pub struct Rtattr {
    pub rta_len: u16,
    pub rta_type: u16,
}


fn addattr_ptr(ptr: *const libc::c_void, offset: usize,
               rta_type: libc::c_int, rta_len: usize, src_ptr: *const libc::c_void, alen: usize) {
    unsafe {
        let rta = ptr.offset(offset as isize) as *mut Rtattr;
        let rta_ptr = rta as *const _ as *mut libc::c_void;
        let dst_ptr = rta_ptr.offset(size_of::<Rtattr>() as isize);

        (*rta).rta_len = rta_len as u16;
        (*rta).rta_type = rta_type as u16;

        copy(src_ptr, dst_ptr, alen);
    }
}

pub fn addattr_l(h: &mut Nlmsghdr, maxlen: usize, rta_type: libc::c_int, src: &[u8], alen: usize) -> bool {
    let rta_len = rta_length(alen);
    let offset = nlmsg_align(h.nlmsg_len as usize);

    if offset + rta_len > maxlen {
        false
    } else {
        let ptr = h as *const _ as *const libc::c_void;
        let src_ptr = src as *const _ as *const libc::c_void;

        addattr_ptr(ptr, offset, rta_type, rta_len, src_ptr, alen);
        h.nlmsg_len = (offset + rta_len) as u32;

        true
    }
}

/*
pub fn rta_addattr_l(rta: &mut Rtattr, maxlen: usize, rta_type: libc::c_int, src: &[u8], alen: usize) -> bool {
    let rta_len = rta_length(alen);
    let offset = rta_align(rta.rta_len as usize);

    if offset + rta_len > maxlen {
        false
    } else {
        let ptr = rta as *const _ as *const libc::c_void;
        let src_ptr = src as *const _ as *const libc::c_void;

        addattr_ptr(ptr, offset, rta_type, rta_len, src_ptr, alen);
        rta.rta_len = (offset + rta_len) as u16;

        true
    }
}
*/

pub fn addattr32(h: &mut Nlmsghdr, maxlen: usize, rta_type: libc::c_int, src: u32) -> bool {
    let rta_len = rta_length(size_of::<u32>());
    let offset = nlmsg_align(h.nlmsg_len as usize);

    if offset + rta_len > maxlen {
        false
    } else {
        let ptr = h as *const _ as *const libc::c_void;
        let src_ptr = &src as *const _ as *mut libc::c_void;

        addattr_ptr(ptr, offset, rta_type, rta_len, src_ptr, size_of::<u32>());
        h.nlmsg_len = (offset + rta_len) as u32;

        true
    }
}

pub fn rta_addrtnh(rta: &mut Rtattr, maxlen: usize, rta_type: libc::c_int, src: &[u8], alen: usize) -> bool {
    let rta_len = rta_length(alen);
    let rtnh_len = size_of::<Rtnexthop>();
    let mut offset = rta_align(rta.rta_len as usize);

    if offset + rta_len + rtnh_len > maxlen {
        false
    } else {
        let ptr = rta as *const _ as *const libc::c_void;
        let src_ptr = src as *const _ as *const libc::c_void;

        offset += rtnh_len;

        unsafe {
            let mut rtnh = ptr.offset(rta.rta_len as isize) as *mut Rtnexthop;

            rta.rta_len += rtnh_len as u16;

            addattr_ptr(ptr, offset, rta_type, rta_len, src_ptr, alen);
            rta.rta_len = (offset + rta_len) as u16;

            (*rtnh).rtnh_len = (rtnh_len + alen + size_of::<Rtattr>()) as u16;
            (*rtnh).rtnh_flags = 0;
            (*rtnh).rtnh_hops = 0;
        }

        true
    }
}


const RTNH_ALIGNTO: usize = 4usize;

/// struct rtnexthop
#[repr(C)]
pub struct Rtnexthop {
    pub rtnh_len: u16,
    pub rtnh_flags: u8,
    pub rtnh_hops: u8,
    pub rtnh_ifindex: i32,
}

pub fn rtnh_align(len: usize) -> usize {
    (len + RTNH_ALIGNTO - 1) & !(RTNH_ALIGNTO - 1)
}

pub fn rtnh_length(len: usize) -> usize {
    rtnh_align(size_of::<Rtnexthop>()) + len
}

