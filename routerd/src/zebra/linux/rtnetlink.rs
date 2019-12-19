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

