//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Nexus Message
// - Nexus to Protocol
//   - Timer Expiration
//   - Config Request
//   - Exec Reqeust
//   - Protocol Termination
//
// - Protocol to Nexus
//   - Timer Registration
//   - Config Response
//   - Exec Response
//

use std::time::Duration;

use common::method::Method;

use crate::core::protocols::ProtocolType;

/// Message from Nexus to Protocol.
pub enum NexusToProto {
    /// Notify timer expiration.
    ///   Nexus notifies timer expiration to registered protocol.
    ///     u32: Token
    TimerExpiration(u32),

    /// Config Request
    ///   Request to add/delete/update configuration to protocol.
    ///     u32: Client id(inferred from UdsServerEntry.index)
    ///     Method: method
    ///     String: path
    ///     Value: JSON object in String
    ConfigRequest((u32, Method, String, Option<Box<String>>)),

    /// Exec Request
    ///   Request to execute control command
    ///     u32: Client id(inferred from UdsServerEntry.index)
    ///     Method: method
    ///     String: path
    ///     Value: JSON object in String
    ExecRequest((u32, Method, String, Option<Box<String>>)),

    /// Notify protocol termination.
    ///   Nexus requests protocol to terminate.
    ProtoTermination,
}

impl Clone for NexusToProto {
    fn clone(&self) -> Self {
        match self {
            NexusToProto::TimerExpiration(v) =>
                NexusToProto::TimerExpiration(*v),
            NexusToProto::ConfigRequest((i, m, s, opt)) =>
                 NexusToProto::ConfigRequest((*i, m.clone(), s.clone(), opt.clone())),
            NexusToProto::ExecRequest((i, m, s, opt)) =>
                 NexusToProto::ExecRequest((*i, m.clone(), s.clone(), opt.clone())),
            NexusToProto::ProtoTermination =>
                NexusToProto::ProtoTermination
        }
    }
}

/// Message from Protocol to Nexus.
pub enum ProtoToNexus {
    /// Register timer to server.
    ///   Protocol registers timer to Nexus.
    ///     ProtocolType: Type of protocol
    ///     Duration: Time to expire
    ///     u32: Token
    TimerRegistration((ProtocolType, Duration, u32)),

    /// Config Response.
    ///   Response for configuration being applied.
    ///     u32: Client id
    ///     String: JSON format.
    ConfigResponse((u32, Option<Box<String>>)),

    /// Exec Response.
    ///   Response for control command output.
    ///     u32: Client id
    ///     String: JSON format.
    ExecResponse((u32, Option<Box<String>>)),

    /// Register config to nexus.
    ///   Protocol registers config path to Nexus
    ///   Nexus sends config update asynchronously through PostConfig message.
    ///     ProtocolType: Type of protocol
    ///     String: path
    ///     bool: whether or not sends current configs in bulk.
    // ConfigRegistration((ProtocolType, String, bool)),

    /// Notify protocol exception to Nexus.
    /// ???
    ProtoException(String),
}
