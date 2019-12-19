//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra Message
// - ProtoToZebra
//
// - ZebraToProto
//

use std::sync::mpsc;

use crate::core::protocols::ProtocolType;

pub enum ProtoToZebra {
    // Register ZebraToProto channel
    RegisterProto((ProtocolType, mpsc::Sender<ZebraToProto>)),

    RouteAdd(i32),
    RouteLookup(i32)
}

pub enum ZebraToProto {
    Interface(i32),
    InterfaceAddr(i32),
    InterfaceState(i32),
    Route(i32)
}
