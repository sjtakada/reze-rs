//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - IPv4/IPv6 RIB manager, Kernel Interface.
//

pub mod error;
pub mod master;
pub mod link;
pub mod address;   
pub mod route;
pub mod rib;
pub mod static_route;

pub mod kernel;

#[cfg(target_os = "linux")]
pub mod linux;
