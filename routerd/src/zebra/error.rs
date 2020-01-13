//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Zebra Error
//

use quick_error::*;

quick_error! {
    #[derive(Debug)]
    pub enum ZebraError {
        Other(s: String) {
            description("Other error")
            display(r#"Other error {}"#, s)
        }
        System(s: String) {
            description("System error")
            display(r#"System error {}"#, s)
        }
        Route(s: String) {
            description("Route error")
            display(r#"Route error {}"#, s)
        }
        Link(s: String) {
            description("Link error")
            display(r#"Link error {}"#, s)
        }
        Address(s: String) {
            description("Address error")
            display(r#"Address error {}"#, s)
        }
        Encode(s: String) {
            description("Encode error")
            display(r#"Encode error {}"#, s)
        }
    }
}

