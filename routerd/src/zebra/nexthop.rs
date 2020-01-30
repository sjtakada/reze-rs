//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra - Nexthop.
//

use std::fmt;

use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

use rtable::prefix::*;

/// Nexthop.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Nexthop<T: Addressable> {

    /// IP Address.
    Address(T),

    /// Interface Name.
    Ifname(String),

    /// Network Prefix - TBD: floating nexthop.
    Network(Prefix<T>),
}

/// Serializer for Nexthop
impl<T> Serialize for Nexthop<T>
where T: Addressable
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        let mut s = serializer.serialize_struct("Nexthop", 1)?;
        let str = match self {
            Nexthop::<T>::Address(addr) => addr.to_string(),
            _ => "()".to_string(),
        };

        s.serialize_field("nexthop", &str)?;
        s.end()
    }
}

impl<T> Nexthop<T>
where T: Addressable
{
    /// Construct Nexthop from IP address.
    pub fn from_address(address: &T) -> Nexthop<T> {
        Nexthop::<T>::Address(address.clone())
    }

    /// Construct Nexthop from IP address string.
    pub fn from_address_str(s: &str) -> Option<Nexthop<T>> {
        match T::from_str(s) {
            Ok(address) => Some(Nexthop::<T>::Address(address.clone())),
            Err(_) => None,
        }
    }

    /// Construct Nexthop from Interface name.
    pub fn from_ifname(ifname: &str) -> Nexthop<T> {
        Nexthop::<T>::Ifname(String::from(ifname))
    }
}

impl<T> fmt::Display for Nexthop<T>
where T: Addressable
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Nexthop::<T>::Address(address) => {
                write!(f, "{:?}", address)
            },
            Nexthop::<T>::Ifname(ifname) => {
                write!(f, "{}", ifname)
            },
            Nexthop::<T>::Network(prefix) => {
                write!(f, "{:?}", prefix)
            },
        }
    }
}

impl<T> fmt::Debug for Nexthop<T>
where T: Addressable
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Nexthop::<T>::Address(address) => {
                write!(f, "{:?}", address)
            },
            Nexthop::<T>::Ifname(ifname) => {
                write!(f, "{}", ifname)
            },
            Nexthop::<T>::Network(prefix) => {
                write!(f, "{:?}", prefix)
            },
        }
    }
}
