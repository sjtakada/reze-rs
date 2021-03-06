//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Error
//

use quick_error::*;

quick_error! {
    #[derive(Debug)]
    pub enum CliError {
        InitModeError {
            description("Init mode error")
            display(r#"Could not initialize CLI modes"#)
        }
        SetModeError(mode: String) {
            description("Set mode error")
            display(r#"Could not set CLI mode '{}'"#, mode)
        }
        ConnectError {
            description("Connect error")
            display(r#"Could not connect to the server"#)
        }
        CommandNotFound(s: String) {
            description("Command not found")
            display(r#"The command "{}" could not be found"#, s)
        }
        //
        ActionError(s: String) {
            description("Action error")
            display(r#"Could not handle action {}"#, s)
        }
        NoActionDefined {
            description("No action defined")
            display(r#"No action defined"#)
        }
        ChildProcessError {
            description("Child process execution error")
            display(r#"Child process execution error"#)
        }
        RemoteSendError {
            description("Remote send error")
            display(r#"Remote send error"#)
        }
        RemoteReceiveError {
            description("Remote receive error")
            display(r#"Remote receive error"#)
        }
    }
}
