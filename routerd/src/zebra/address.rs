//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - IPv4 and IPv6 address handler.
//

use rtable::prefix::*;

use super::kernel::KernelAddr;

/// Connected Address.
pub struct Connected<T: Addressable> {

    /// Address prefix.
    address: Prefix<T>,

    /// Destination address prefix for peer.
    destination: Option<Prefix<T>>,

    /// Secondary address.
    secondary: bool,

    /// Unnumbered.
    unnumbered: bool,

    /// Label.
    label: Option<String>,
}

/// Connected implementation.
impl<T: Addressable> Connected<T> {

    /// Constructor.
    pub fn new(prefix: Prefix<T>) -> Connected<T> {
        Connected::<T> {
            address: prefix,
            destination: None,
            secondary: false,
            unnumbered: false,
            label: None,
        }
    }

    /// Construct from kernel.
    pub fn from_kernel(ka: KernelAddr<T>) -> Connected<T> {
        Connected::<T> {
            address: ka.address,
            destination: ka.destination,
            secondary: ka.secondary,
            unnumbered: ka.unnumbered,
            label: ka.label,
        }
    }

    /// Return address prefix.
    pub fn address(&self) -> &Prefix<T> {
        &self.address
    }

    ///
    pub fn destination(&self) -> &Option<Prefix<T>> {
        &self.destination
    }

    pub fn secondary(&self) -> bool {
        self.secondary
    }

    pub fn unnumbered(&self) -> bool {
        self.unnumbered
    }

    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }
}
