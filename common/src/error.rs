//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Error
//

use quick_error::*;

quick_error! {
    #[derive(Debug)]
    pub enum CoreError {
        GenericError(s: String) {
            description("Generic Error")
            display(r#"{}"#, s)
        }
        SystemShutdown {
            description("System shutdown")
            display(r#"System shutdown"#)
        }
        UnknownEvent {
            description("Unknown event")
            display(r#"Unknown event"#)
        }
        UdsConnectError {
            description("UDS connect error")
            display(r#"UDS connect error"#)
        }
        UdsWriteError {
            description("UDS write error")
            display(r#"UDS write error"#)
        }
        ChannelSendError(s: String) {
            description("Channel send error")
            display(r#"Channel write error {}"#, s)
        }
        ChannelNoSender {
            description("Channel sender does not exist")
            display(r#"Channel sender does not exist"#)
        }
        ChannelQueueEmpty {
            description("Channel queue is empty")
            display(r#"Channel queue is empty"#)
        }
        RequestInvalid(s: String) {
            description("Command request is invalid")
            display(r#"Command request {} is invalid"#, s)
        }
        MdsNoHandler(s: String) {
            description("Mds handler does not exist")
            display(r#"Mds handler does not exist in {}"#, s)
        }
        NotImplemented {
            description("Trait function not implemented")
            display(r#"Trait function not implemented"#)
        }
        ConfigNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
        CommandExec(s: String) {
            description("Command execution error")
            display(r#"Command execution error {}"#, s)
        }
        NexusToProto {
            description("Sending message from Nexus to Protocol")
            display(r#"Sending message from Nexus to Protocol"#)
        }
        ParseMethod {
            description("Unknown Method")
            display(r#"Uknown Method"#)
        }
    }
}

/// Utility.
impl CoreError {
    pub fn json_status(&self) -> String {
        format!("{{\"status\":\"Error\",\"message\":\"{}\"}}", self.to_string())
    }
}
