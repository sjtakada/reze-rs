//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Zebra Message
//

pub enum ProtoToZebra {
    RouteAdd(i32),
    RouteLookup(i32)
}

pub enum ZebraToProto {
    Interface(i32),
    InterfaceAddr(i32),
    InterfaceState(i32),
    Route(i32)
}
