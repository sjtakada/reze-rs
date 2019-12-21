//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Core Error
//

use quick_error::*;

quick_error! {
    #[derive(Debug)]
    pub enum CoreError {
        NexusTermination {
            description("Nexus is terminated")
            display(r#"Nexus is terminated"#)
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

