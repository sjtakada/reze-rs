//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Protocols.
//

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
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
