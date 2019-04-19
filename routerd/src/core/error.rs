//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Error
//

use quick_error::*;

quick_error! {
    #[derive(Debug)]
    pub enum CoreError {
        NexusTermination {
            description("Nexus is terminated")
            display(r#"Nexus is terminated"#)
        }
        CommandNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
    }
}

