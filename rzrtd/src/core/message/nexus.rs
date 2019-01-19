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

// Message from Protocol to Nexus.
pub enum ProtoToNexus {
    // Register timer to server.
    TimerRegistration((ProtocolType, Duration, u32)),

    // Notify protocol exception to Nexus.
    ProtoException(String),
}

// Message from Nexus to Protocol.
pub enum NexusToProto {
    // Notify timer expiration.
    TimerExpiration(u32),

    // Send configuration.
    PostConfig((String, Vec<String>)),

    // Notify protocol termination.
    ProtoTermination,
}

