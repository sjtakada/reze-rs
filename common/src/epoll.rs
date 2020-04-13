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

pub type EpollEvent = libc::epoll_event;

/// Epoll EventManager
pub struct EpollEventManager {

    /// Reception vector to store ready events.
    events: Vec<EpollEvent>,

    /// Epoll FD.
    epoll_fd: RawFd,

    /// Waiting tasks.
    task_waiting: HashMap<RawFd, Arc<Task>>,

    /// Ready tasks
    task_ready: Vec<Arc<Task>>,
}

impl EpollEventManager {

    /// Constructor.
    pub fn new() -> io::Result<EpollEventManager> {
        let epoll_fd = syscall!(epoll_create1(libc::O_CLOEXEC))?;

        Ok(EpollEventManager {
            events: Vec::with_capacity(1024),
            epoll_fd: epoll_fd,
            task_waiting: HashMap::new(),
            task_ready: Vec::new(),
        })
    }

    /// Register.
    pub fn register_read(&mut self, fd: RawFd, task: Arc<Task>) {
        self.task_waiting.insert(fd, task.clone());

        let mut event = libc::epoll_event {
            events: (EPOLLET | EPOLLIN | EPOLLRDHUP) as u32,
            u64: fd as u64,
        };

        syscall!(epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event));
    }

    /// Pop ready task.
    pub fn pop_ready(&mut self) -> Option<Arc<Task>> {
        self.task_ready.pop()
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
                        let task = self.task_waiting.remove(&fd).unwrap();
                        self.task_ready.push(task);
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
//    event_manager: Arc<EventManager>,

    /// Raw FD.
    fd: RawFd,
}

impl EpollFuture {

    /// Constructor.
    pub fn new(/*event_manager: Arc<EventManager>, */fd: RawFd) -> Self {
        EpollFuture {
//            event_manager: event_manager,
            fd: fd,
        }
    }
}

impl Future for EpollFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(())
    }
}
