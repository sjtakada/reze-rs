//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Address Famliy
//

use std::net::{Ipv4Addr, Ipv6Addr};

/// AddressFamily trait.
///   Abstract net::Ipv4Addr and net::Ipv6Addr.
pub trait AddressFamily {

    /// Return libc Addressfamily
    fn address_family() -> libc::c_int;
}

/// AddressFamily for Ipv4Addr.
impl AddressFamily for Ipv4Addr {
    fn address_family() -> libc::c_int {
        libc::AF_INET
    }
}

/// AddressFamily for Ipv6Addr.
impl AddressFamily for Ipv6Addr {
    fn address_family() -> libc::c_int {
        libc::AF_INET6
    }
}

