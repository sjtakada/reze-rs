//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Encode:
//  Low level utility functions to set/get arbitrary value into/from buffer.
//  with unsafe operation.  All integer values are host byte order.
//

use std::ops::AddAssign;
use std::mem::size_of;
use std::ptr::copy;

//use super::super::error::ZebraError;

/*
pub fn set_data_unsafe(buf: &mut [u8], pos: usize, data: &[u8], len: usize) {
    let ptr = buf as *const _ as *mut libc::c_void;
    let src_ptr = data as *const _ as *const libc::c_void;

    unsafe {
        let dst_ptr = ptr.offset(pos as isize);

        copy(src_ptr, dst_ptr, len);
    }
}
*/

pub fn encode_data(buf: &mut [u8], data: &[u8]) {
    let dst = &mut buf[..data.len()];

    dst.copy_from_slice(data);
}

pub fn encode_num<T>(buf: &mut [u8], v: T) {
    let ptr = buf as *const _ as *const libc::c_void;

    unsafe {
        let dst = ptr as *mut T;
        (*dst) = v;
    }
}

/*
pub fn add_num<T: AddAssign>(buf: &mut [u8], pos: usize, v: T) {
    let ptr = buf as *const _ as *const libc::c_void;

    unsafe {
        let dst = ptr.offset(pos as isize) as *mut T;
        (*dst) += v;
    }
}

/// Set T value to buf.
pub fn encode_num<T>(buf: &mut [u8], val: T) -> Result<usize, ZebraError>
{
    let len = size_of::<T>();

    if len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // Set value.
        set_num::<T>(buf, val);

        Ok(len)
    }
}

/// Set data to buf.
pub fn encode_data(buf: &mut [u8], val: &[u8]) -> Result<usize, ZebraError>
{
    let len = val.len();

    if len > buf.len() {
        Err(ZebraError::Encode("buffer overflow".to_string()))
    } else {
        // Set value.
        set_data(buf, val);

        Ok(len)
    }
}

*/
