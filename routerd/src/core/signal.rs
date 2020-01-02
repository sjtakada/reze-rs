//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// UNIX Signal.
//

use std::sync;

use nix::sys::signal;

static SIGTSTP_ONCE: sync::Once = sync::Once::new();
static SIGINT_CAUGHT: sync::atomic::AtomicUsize = sync::atomic::AtomicUsize::new(0);

extern fn sigint_handler(_: i32) {
    SIGINT_CAUGHT.fetch_add(1, sync::atomic::Ordering::SeqCst);
}

pub fn is_sigint_caught() -> bool {
    SIGINT_CAUGHT.load(sync::atomic::Ordering::SeqCst) > 0
}

pub fn signal_init() {
    SIGTSTP_ONCE.call_once(|| unsafe {
        let sa = signal::SigAction::new(
            signal::SigHandler::Handler(sigint_handler),
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        let _ = signal::sigaction(signal::SIGINT, &sa);
    });
}
