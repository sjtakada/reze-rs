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
        SystemShutdown {
            description("System shutdown")
            display(r#"System shutdown"#)
        }
        UdsConnectError {
            description("UDS connect error")
            display(r#"UDS connect error"#)
        }
        RequestInvalid(s: String) {
            description("Command request is invalid")
            display(r#"Command request {} is invalid"#, s)
        }
        ConfigNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
        CommandExec(s: String) {
            description("Command execution error")
            display(r#"Command execution error {}"#, s)
        }
        ParseMethod {
            description("Unknown Method")
            display(r#"Uknown Method"#)
        }
    }
}

