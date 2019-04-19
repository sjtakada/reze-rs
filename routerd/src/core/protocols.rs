//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Protocols.
//

use std::fmt;

#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ProtocolType {
    Master,
    Zebra,
    Rip,
    Ripng,
    Ospf,
    OspfV3,
    Isis,
    Eigrp,
    Bgp,
    Vrrp,
    Nhrp,
}

// TBD: impl Display fro ProtocolType
impl fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ProtocolType::Master => "Master",
            ProtocolType::Zebra => "Zebra",
            ProtocolType::Rip => "RIP",
            ProtocolType::Ripng => "RIPng",
            ProtocolType::Ospf => "OSPF",
            ProtocolType::OspfV3 => "OSPFv3",
            ProtocolType::Isis => "IS-IS",
            ProtocolType::Eigrp => "EIGRP",
            ProtocolType::Bgp => "BGP",
            ProtocolType::Vrrp => "VRRP",
            ProtocolType::Nhrp => "NHRP",
        };

        write!(f, "{}", s)
    }
}
