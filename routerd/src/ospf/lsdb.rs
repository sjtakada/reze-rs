//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF Link State Database
//

use std::fmt;
use std::cmp::Ordering;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::error::Error;

use rtable::prefix::*;
use rtable::tree::*;

const LS_BIT_LENGTH: u8 = 64;

/// Link State ID and Advertising Router tuple for LSDB index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LsTuple {

    /// Link State ID.
    id: Ipv4Addr,

    /// Advertising Router.
    adv_router: Ipv4Addr,
}

impl Addressable for LsTuple {

    /// Return address bit length.
    fn bit_len() -> u8 {
        LS_BIT_LENGTH
    }

    /// Construct address with all 0s.
    fn empty_new() -> Self {
        LsTuple {
            id: Ipv4Addr::UNSPECIFIED,
            adv_router: Ipv4Addr::UNSPECIFIED,
        }
    }

    /// Construct address from slice.
    fn from_slice(s: &[u8]) -> Self {
        let t1: [u8; 4] = [s[0], s[1], s[2], s[3]];
        let t2: [u8; 4] = [s[4], s[5], s[6], s[7]];

        LsTuple {
            id: Ipv4Addr::from(t1),
            adv_router: Ipv4Addr::from(t2),
        }
    }

    /// Return reference of slice to address.
    fn octets_ref(&self) -> &[u8] {
        let p = (self as *const LsTuple) as *const u8;
        unsafe {
            std::slice::from_raw_parts(p, std::mem::size_of::<LsTuple>())
        }
    }
}

impl fmt::Display for LsTuple {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}:{}", self.id, self.adv_router)
    }
}

impl fmt::Debug for LsTuple {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id={}, adv_router={}", self.id, self.adv_router)
    }
}

impl FromStr for LsTuple {
    type Err = std::net::AddrParseError;

    /// "A.B.C.D:A.B.C.D".
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split(':').collect();
        let id = v[0].parse()?;
        let adv_router = v[1].parse()?;
        
        Ok(LsTuple { id, adv_router })
    }
}

pub struct OspfLsdb {

}
