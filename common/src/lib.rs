//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Common library
//

#[cfg(unix)]
macro_rules! syscall {
    ($func:ident ( $($arg:expr),* $(,)* ) ) => ({
        let ret = unsafe { libc::$func($($arg, )*) };
        if ret < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(ret)
        }
    });
}

pub mod consts;
pub mod error;
pub mod event;
pub mod timer;
pub mod method;
pub mod uds_server;
pub mod uds_client;
pub mod acl;

#[cfg(any(target_os = "linux"))]
pub mod epoll;


