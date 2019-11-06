//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Nexus Message
// - Nexus to Protocol
//   - Timer Expiration
//   - Post Configuration 
//   - Show Command (sync)
// - Protocol to Nexus
//   - Timer Registration
//   - Config Registration
//   - Show Command output
//   - Protocol Termination
//

use std::time::Duration;

use crate::core::protocols::ProtocolType;
use crate::core::config::Method;

/// Message from Nexus to Protocol.
pub enum NexusToProto {
    /// Notify timer expiration.
    ///   Nexus notifies timer expiration to registered protocol.
    ///     u32: Token
    TimerExpiration(u32),

    /// Send configuration.
    ///   Nexus sends configuration.
    ///     Method: method
    ///     String: path
    ///     Value: JSON object in String
    SendConfig((Method, String, Box<String>)),

    /// Notify protocol termination.
    ///   Nexus requests protocol to terminate.
    ProtoTermination,
}

/// Message from Protocol to Nexus.
pub enum ProtoToNexus {
    /// Register timer to server.
    ///   Protocol registers timer to Nexus.
    ///     ProtocolType: Type of protocol
    ///     Duration: Time to expire
    ///     u32: Token
    TimerRegistration((ProtocolType, Duration, u32)),

    /// Register config to nexus.
    ///   Protocol registers config path to Nexus
    ///   Nexus sends config update asynchronously through PostConfig message.
    ///     ProtocolType: Type of protocol
    ///     String: path
    ///     bool: whether or not sends current configs in bulk.
    // ConfigRegistration((ProtocolType, String, bool)),

    /// Request configuration.
    //ConfigReqeust(String),

    /// Notify protocol exception to Nexus.
    /// ???
    ProtoException(String),
}
