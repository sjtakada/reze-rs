//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF Area
//

use std::net::Ipv4Addr;

use super::consts::OspfAuth;

/// OSPF Area.
pub struct OspfArea {

    /// Area ID.
    area_id: Ipv4Addr,
}

/// OSPF Area Config.
pub struct OspfAreaConfig {

    /// No summary.
    no_summary: bool,

    /// Stub Default cost.
    default_cost: u32,

    /// Authentication Type.
    auth_type: OspfAuth,
}
