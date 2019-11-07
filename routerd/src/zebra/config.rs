//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - Config.
//

use std::rc::Rc;

use crate::core::config::Config;
use crate::core::config_master::*;
use super::static_route::*;

pub fn zebra_config_init(config: &mut ConfigMaster) {
    let ipv4_routes = Ipv4StaticRoute::new();
//    config.register_child(Rc::new(ipv4_routes));
}
