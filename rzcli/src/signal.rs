//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// UNIX Signal.
//

use std::sync;
use nix::sys::signal;

static SIGTSTP_ONCE: sync::Once = sync::ONCE_INIT;

pub fn ignore_sigtstp_handler() {
    SIGTSTP_ONCE.call_once(|| unsafe {
        let sa = signal::SigAction::new(
            signal::SigHandler::SigIgn,
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        let result = signal::sigaction(signal::SIGTSTP, &sa);

        match result {
            Ok(sa) if sa.handler() != signal::SigHandler::SigIgn => {
                let new_sa = signal::SigAction::new(
                    signal::SigHandler::SigIgn,
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

