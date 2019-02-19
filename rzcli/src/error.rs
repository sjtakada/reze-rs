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
        ConnectError {
            description("Could not connect to the server")
            display(r#"Could not connect to the server"#)
        }
        CommandNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
    }
}
