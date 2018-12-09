//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Protocols.
//

#[derive(Clone)]
pub enum ProtocolType {
    Master,
    Zebra,
    Rip,
    Ripng,
    Ospf,
    Ospfv3,
    Isis,
    Eigrp,
    Bgp,
    Vrrp,
    Nhrp,
}
