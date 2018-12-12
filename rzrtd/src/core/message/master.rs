//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Master Message
// - Master to Protocol
//   - Timer Expiration
//   - Config Command (async)
//   - Show Command (sync)
// - Protocol to Master
//   - Timer Registration
//   - Show Command output
//   - Protocol Termination
//

use std::time::Duration;

use super::super::protocols::ProtocolType;

pub enum ProtoToMaster {
    TimerRegistration((ProtocolType, Duration, i32)),
    ProtoTermination(i32)
}

pub enum MasterToProto {
    TimerExpiration(i32),
    PostConfig((String, Vec<String>))
}

