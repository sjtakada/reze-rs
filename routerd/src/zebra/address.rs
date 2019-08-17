//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - IPv4 and IPv6 address handler.
//

use neli::consts::*;

pub trait AddressHandler {
    // Get all addresses from kernel
    fn get_addresses_all(&self, family: RtAddrFamily) -> Vec<Connected>;
}


pub struct Connected {
    // Address prefix.
    // 
}

impl Connected {
    pub fn new() -> Connected {
        Connected {
        }
    }
}
