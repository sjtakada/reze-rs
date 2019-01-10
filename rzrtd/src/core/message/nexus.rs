//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Nexus Message
// - Nexus to Protocol
//   - Timer Expiration
//   - Config Command (async)
//   - Show Command (sync)
// - Protocol to Nexus
//   - Timer Registration
//   - Show Command output
//   - Protocol Termination
//

use std::time::Duration;

use crate::core::protocols::ProtocolType;

pub enum ProtoToNexus {
    TimerRegistration((ProtocolType, Duration, u32)),
    ProtoTermination(i32)
}

pub enum NexusToProto {
    TimerExpiration(u32),
    PostConfig((String, Vec<String>))
}

