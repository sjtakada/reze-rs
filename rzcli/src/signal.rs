//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// UNIX Signal.
//

use nix;
use nix::sys::signal;

use std::sync;

static SIGTSTP_ONCE: sync::Once = sync::ONCE_INIT;

use nix::sys::signal::SigHandler;

pub fn ignore_sigtstp_handler() {
    SIGTSTP_ONCE.call_once(|| unsafe {
        let sa = signal::SigAction::new(
            SigHandler::SigIgn,
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        let result = signal::sigaction(signal::SIGTSTP, &sa);

        match result {                                                                                                     
            Ok(sa) if sa.handler() != SigHandler::SigIgn => {                                                              
                let new_sa = signal::SigAction::new(                                                                       
                    SigHandler::SigIgn,
                    signal::SaFlags::empty(),                                                                              
                    signal::SigSet::empty()                                                                                
                );                                                                                                         
                let _ = signal::sigaction(signal::SIGTSTP, &new_sa);                                                       
            }                                                                                                              
            _ => {                                                                                                         
                
            }                                                                                                              
        }                                                                                                                  
    });
}

