//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
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
    }
}
