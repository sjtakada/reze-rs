//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF process
//

use std::net::Ipv4Addr;

use super::consts::*;
use super::lsdb::OspfLsdb;

type OspfProcessId = u16;

/// OSPF process.
pub struct Ospf {

    /// Process ID.
    process_id: OspfProcessId,

    /// Router ID.
    router_id: Ipv4Addr,

    /// Up flag.
    up: bool,

    /// ABR flag.
    abr: bool,

    /// ASBR flag.
    asbr: bool,

    /// AS-Scoped LSDB.
    lsdb: OspfLsdb,

    /// OSPF config.
    config: OspfConfig,
}

impl Ospf {

    /// Constructor.
    pub fn new(process_id: OspfProcessId) -> Ospf {
        Ospf {
            process_id: process_id,
            router_id: Ipv4Addr::UNSPECIFIED,
            up: false,
            abr: false,
            asbr: false,
            lsdb: OspfLsdb::new(OspfFloodingScope::As),
            config: OspfConfig::new(),
        }
    }
}

/// OSPF config.
pub struct OspfConfig {

    /// Static Router ID.
    router_id: Ipv4Addr,

    /// RFC1583 compatiblity.
    rfc1583_compat: bool,

    /// Log Adjacency Changes.
    log_adjacency_changes: bool,

    /// SPF delay time.
    spf_delay: u32,

    /// SPF hold time.
    spf_holdtime: u32,

    /// Reference bandwidth.
    ref_bandwidth: u32,
}

impl OspfConfig {

    /// Constructor.
    pub fn new() -> OspfConfig {
        OspfConfig {
            router_id: Ipv4Addr::UNSPECIFIED,
            rfc1583_compat: false,
            log_adjacency_changes: false,
            spf_delay: 10,
            spf_holdtime: 20,
            ref_bandwidth: 100,
        }
    }
}

/// OSPF statstics.
pub struct OspfStats {

    /// Number of LSA originated.
    lsa_originated: usize,

    /// Number of LSA received.
    lsa_received: usize,
}
