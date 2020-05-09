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
use std::collections::HashMap;

use rtable::prefix::*;
use rtable::tree::*;

use super::consts::*;
use super::lsa::OspfLsa;

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

/// Get 4 u8 values from slices and return u32 in network byte order.                               
fn slice_get_u32(s: &[u8], i: usize) -> u32 {
    ((s[i] as u32) << 24) | ((s[i + 1] as u32) << 16) | ((s[i + 2] as u32) << 8) | s[i + 3] as u32
}

/// Copy u32 value to slice.                                                                        
fn slice_copy_u32(s: &mut [u8], v: u32, i: usize) {
    s[i + 0] = ((v >> 24) & 0xFF) as u8;
    s[i + 1] = ((v >> 16) & 0xFF) as u8;
    s[i + 2] = ((v >> 8) & 0xFF) as u8;
    s[i + 3] = (v & 0xFF) as u8;
}

/// Ls Prefix.
pub struct LsPrefix {

    /// LsTuple.
    address: LsTuple,

    /// Prefix length.
    len: u8,
}

impl Prefixable for LsPrefix {

    /// Construct a prefix from given prefix.                                                       
    fn from_prefix(p: &Self) -> Self {
        Self {
            address: p.address.clone(),
            len: p.len
        }
    }

    /// Construct a prefix from common parts of two prefixes, assuming p1 is shorter than p2.       
    fn from_common(prefix1: &Self, prefix2: &Self) -> Self {
        let p1 = prefix1.octets();
        let p2 = prefix2.octets();
        let mut i = 0u8;
        let mut j = 0u8;
        let mut pcommon = Self { address: LsTuple::empty_new(), len: 0 };
        let px = pcommon.octets_mut();
        let bytes = LsTuple::bit_len() / 8;

        while i < bytes {
            let l1: u32 = slice_get_u32(p1, i as usize);
            let l2: u32 = slice_get_u32(p2, i as usize);
            let cp: u32 = l1 ^ l2;
            if cp == 0 {
                slice_copy_u32(px, l1, i as usize);
            }
            else {
                j = cp.leading_zeros() as u8;
                let (mask, _) = match j {
                    0 => (0, false),
                    _ => 0xFFFFFFFFu32.overflowing_shl((32 - j) as u32),
                };
                let v = l1 & (mask as u32);

                slice_copy_u32(px, v, i as usize);
                break;
            }

            i += 4;
        }

        pcommon.len = if prefix2.len() > i * 8 + j {
            i * 8 + j
        } else {
            prefix2.len()
        };

        pcommon
    }

    /// Return prefix length.                                                                       
    fn len(&self) -> u8 {
        self.len
    }

    /// Return reference of slice to address.                                                       
    fn octets(&self) -> &[u8] {
        let p = (&self.address as *const LsTuple) as *const u8;
        unsafe {
            std::slice::from_raw_parts(p, std::mem::size_of::<LsTuple>())
        }
    }

    /// Return mutable reference of slice to address.                                               
    fn octets_mut(&mut self) -> &mut [u8] {
        let p = (&mut self.address as *mut LsTuple) as *mut u8;
        unsafe {
            std::slice::from_raw_parts_mut(p, std::mem::size_of::<LsTuple>())
        }
    }

}

/// Link State Database Info.
struct LsdbInfo {

    /// Tree.
    tree: Tree<LsPrefix, OspfLsa>,

    /// Number of self originated LSAs.
    count_self: usize,

    /// Sum of Checksum.
    checksum: i32,
}

impl LsdbInfo {

    pub fn new() -> LsdbInfo {
        LsdbInfo {
            tree: Tree::new(),
            count_self: 0usize,
            checksum: 0i32,
        }
    }
}

/// Link State Database.
pub struct OspfLsdb {

    /// Collection of Lsdb.
    info: HashMap<OspfLsaType, LsdbInfo>,

    /// Total number of LSAs.
    total: usize,
}

impl OspfLsdb {

    pub fn new(scope: OspfFloodingScope) -> OspfLsdb {
        let mut lsdb = OspfLsdb {
            info: HashMap::new(),
            total: 0usize,
        };

        match scope {
            OspfFloodingScope::Link => {
                lsdb.info.insert(OspfLsaType::LinkScopedOpaqueLsa, LsdbInfo::new());
            }
            OspfFloodingScope::Area => {
                lsdb.info.insert(OspfLsaType::RouterLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::NetworkLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::SummaryLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::AsbrSummaryLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::NssaAsExternalLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::AreaScopedOpaqueLsa, LsdbInfo::new());
            }
            OspfFloodingScope::As => {
                lsdb.info.insert(OspfLsaType::AsExternalLsa, LsdbInfo::new());
                lsdb.info.insert(OspfLsaType::AsScopedOpaqueLsa, LsdbInfo::new());
            }
        }

        lsdb
    }
}

