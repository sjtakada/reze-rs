//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF - OSPF interface
//

use super::ism::IsmState;

/// OSPF interface.
pub struct OspfInterface {

    /// Interface State.
    pub state: IsmState,
}

unsafe impl Send for OspfInterface {}
unsafe impl Sync for OspfInterface {}
