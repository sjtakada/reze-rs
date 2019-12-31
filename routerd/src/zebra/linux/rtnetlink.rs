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

use rtable::prefix::*;

use super::encode::*;
use super::super::error::ZebraError;

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

pub fn nlmsg_addattr_l(nlmsg_len: &mut u32, buf: &mut [u8],
                       rta_type: i32, data: &[u8], alen: usize) -> Result<usize, ZebraError> {
    let len = addattr_l(buf, rta_type, data, alen)?;

    *nlmsg_len += len as u32;

    Ok(len)
}

/// Add Rtattr with arbitrary data to buffer.
pub fn addattr_l(buf: &mut [u8], rta_type: i32, data: &[u8], alen: usize) -> Result<usize, ZebraError> {
    let rta_len = rta_length(alen);

    if rta_len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // RTA type.
        encode_num::<u16>(&mut buf[2..], rta_type as u16);

        // RTA payload.
        encode_data(&mut buf[4..], data);

        // RTA length.
        encode_num::<u16>(&mut buf[..], rta_len as u16);

        Ok(rta_len)
    }
}

pub fn nlmsg_addattr_payload<F>(nlmsg_len: &mut u32, buf: &mut [u8],
                                rta_type: i32,  encode_payload: F) -> Result<usize, ZebraError>
where F: Fn(&mut [u8]) -> Result<usize, ZebraError>
{
    let len = addattr_payload(buf, rta_type, encode_payload)?;

    *nlmsg_len += len as u32;

    Ok(len)
}

/// Add Rtattr with payload encoded by encoder to buffer.
pub fn addattr_payload<F>(buf: &mut [u8], rta_type: i32, encode_payload: F) -> Result<usize, ZebraError>
where F: Fn(&mut [u8]) -> Result<usize, ZebraError>
{
    let header_len = size_of::<Rtattr>();

    if header_len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // RTA type.
        encode_num::<u16>(&mut buf[2..], rta_type as u16);

        // RTA payload.
        let payload_len = encode_payload(&mut buf[header_len..])?;

        // RTA length.
        encode_num::<u16>(&mut buf[..], (payload_len + header_len) as u16);

        Ok(payload_len + header_len)
    }
}

pub fn nlmsg_addattr32(nlmsg_len: &mut u32, buf: &mut [u8],
                       rta_type: i32, src: u32) -> Result<usize, ZebraError> {
    let len = addattr32(buf, rta_type, src)?;

    *nlmsg_len += len as u32;

    Ok(len)
}

/// Add Rtattr with u32 value to buffer.
pub fn addattr32(buf: &mut [u8], rta_type: libc::c_int, src: u32) -> Result<usize, ZebraError> {
    let rta_len = rta_length(size_of::<u32>());

    if rta_len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // RTA type.
        encode_num::<u16>(&mut buf[2..], rta_type as u16);

        // RTA payload.
        encode_num::<u32>(&mut buf[4..], src);

        // RTA length.
        encode_num::<u16>(&mut buf[..], rta_len as u16);

        Ok(rta_len)
    }
}

const RTNH_ALIGNTO: usize = 4usize;

/// struct rtnexthop from rtnetlink.h.
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |           rtnh_len            |  rtnh_flags   |   rtnh_hops   |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                          rtnh_ifindex                         |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///  |                            Rtattr  ...                        |
///  |                                                               |
///  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///

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

pub fn nlmsg_add_rtnexthop<T: Addressable>(buf: &mut [u8], address: &T) -> Result<usize, ZebraError> {
    let octets: &[u8] = address.octets_ref();
    let rtnh_len = size_of::<Rtnexthop>() + size_of::<Rtattr>() + T::byte_len() as usize;  // XXX probably should align.

    if rtnh_len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // rtnh_len
        encode_num::<u16>(&mut buf[..], 16 as u16);

        // rtnh_flags
        encode_num::<u8>(&mut buf[2..], 0 as u8);

        // rtnh_hops
        encode_num::<u8>(&mut buf[3..], 0 as u8);

        // rtnn_index
        encode_num::<u32>(&mut buf[4..], 0 as u32);

        // RTA Gataway.
        addattr_l(&mut buf[8..], libc::RTA_GATEWAY as i32, &octets[..], T::byte_len() as usize);

        Ok(rtnh_len)
    }
}
