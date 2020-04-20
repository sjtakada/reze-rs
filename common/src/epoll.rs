//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Linux/Epoll
//

use std::io;
use std::os::unix::io::RawFd;
use std::mem::zeroed;
use std::collections::HashMap;
//use std::os::unix::io::AsRawFd;
use std::future::Future;
use std::task::Context;
use std::task::Poll;
use std::pin::Pin;
use std::sync::Arc;

use futures::task::waker_ref;

use libc::{EPOLLIN, EPOLLOUT, EPOLLRDHUP, EPOLLET};

use super::event::Task;
use super::event::EventManager;
use super::event::FutureManager;

pub type EpollEvent = libc::epoll_event;

/// Epoll EventManager
pub struct EpollEventManager {

    /// Epoll FD.
    epoll_fd: RawFd,

    /// Reception vector to store ready events.
    events: Vec<EpollEvent>,

    /// Waiting tasks.
    pub task_waiting: HashMap<RawFd, (bool, Arc<Task>)>,
}

unsafe impl Send for EpollEventManager {}
unsafe impl Sync for EpollEventManager {}

impl EpollEventManager {

    /// Constructor.
    pub fn new() -> io::Result<EpollEventManager> {
        let epoll_fd = syscall!(epoll_create1(libc::O_CLOEXEC))?;

        Ok(EpollEventManager {
            epoll_fd: epoll_fd,
            events: Vec::with_capacity(1024),
            task_waiting: HashMap::new(),
        })
    }

    /// Register.
    pub fn register_read(&mut self, fd: RawFd, task: Arc<Task>) {
        self.task_waiting.insert(fd, (false, task.clone()));

        let mut event = libc::epoll_event {
            events: (EPOLLET | EPOLLIN | EPOLLRDHUP) as u32,
            u64: fd as u64,
        };

        syscall!(epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event));
    }

    /// Pop ready task.
    pub fn pop_ready(&mut self) -> Option<Arc<Task>> {
//        self.task_ready.pop()
        None
    }

    /// Run epoll_wait.
    pub fn wait(&mut self) {
        self.events.clear();
        let timeout = 10;

        match syscall!(epoll_wait(self.epoll_fd, self.events.as_mut_ptr(), self.events.capacity() as i32, timeout)) {
            Ok(num) => {
                unsafe {
                    self.events.set_len(num as usize);

                    for e in &self.events {
                        let fd  = e.u64 as RawFd;
                        let (_, task) = self.task_waiting.remove(&fd).unwrap();
                        self.task_waiting.insert(fd, (true, task));
                    }
                };
            }
            Err(err) => {
            }
        }
    }
}

/// Epoll Future.
pub struct EpollFuture {

    /// EventManager.
    event_manager: Arc<FutureManager>,

    /// Raw FD.
    fd: RawFd,
}

impl EpollFuture {

    /// Constructor.
    pub fn new(event_manager: Arc<FutureManager>, fd: RawFd) -> Self {
        EpollFuture {
            event_manager: event_manager,
            fd: fd,
        }
    }
}

impl Future for EpollFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let event_manager = self.event_manager.clone();
        let mut epoll = event_manager.epoll();

        if let Some((flag, _)) = epoll.task_waiting.get(&self.fd) {
            if *flag {
                epoll.task_waiting.remove(&self.fd);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            Poll::Pending
        }
    }
}
