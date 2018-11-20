//
// ReZe.Rs - Router Daemon
//           BootStrap to initialize basic event handlers.
//
// Copyright (C) 2018 Toshiaki Takada
//

use futures::{Future, Async, Poll};

pub struct BootStrap {

}

impl BootStrap {
    pub fn new() -> BootStrap {
        BootStrap { }
    }
}

impl Future for BootStrap {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::NotReady)
    }
}


