//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Encode:
//  Low level utility functions to set/get arbitrary value into/from buffer.
//  with unsafe operation.  All integer values are host byte order.
//

/// Copy arbitrary number of bytes from a slice to other.
pub fn encode_data(buf: &mut [u8], data: &[u8]) {
    let dst = &mut buf[..data.len()];

    dst.copy_from_slice(data);
}

/// Copy integer value in host byte order.
/// Caller MUST ensure if it does not overflow the buffer.
pub fn encode_num<T>(buf: &mut [u8], v: T) {
    let ptr = buf as *const _ as *const libc::c_void;

    unsafe {
        let dst = ptr as *mut T;
        *dst = v;
    }
}

/// Return integer value in host byte order.
pub fn decode_num<T>(buf: &[u8]) -> T
where T: Copy
{
    unsafe { *(buf as *const _ as *const T) }
}
