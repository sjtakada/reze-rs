//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra - IPv4 and IPv6 address handler.
//

pub trait AddressHandler {
    // Get all addresses from kernel
    fn get_addresses_all(&self, family: libc::c_int) -> Vec<Connected>;
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
