//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF process
//

use std::net::Ipv4Addr;

/// OSPF process.
pub struct Ospf {
    
    /// Router ID.
    router_id: Ipv4Addr,

    /// ABR flag.
    abr: bool,

    /// ASBR flag.
    asbr: bool,
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

/// OSPF statstics.
pub struct OspfStats {

    /// Number of LSA originated.
    lsa_originated: usize,

    /// Number of LSA received.
    lsa_received: usize,
}
