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

    /// Waiting tasks.
    task_waiting: HashMap<RawFd, Arc<Task>>,

    /// Ready status.
    ready: HashMap<RawFd, bool>,
}

unsafe impl Send for EpollEventManager {}
unsafe impl Sync for EpollEventManager {}

impl EpollEventManager {

    /// Constructor.
    pub fn new() -> io::Result<EpollEventManager> {
        let epoll_fd = syscall!(epoll_create1(libc::O_CLOEXEC))?;

        Ok(EpollEventManager {
            epoll_fd: epoll_fd,
            task_waiting: HashMap::new(),
            ready: HashMap::new(),
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

    /// Return vector of waiting tasks.
    pub fn task_collect(&self) -> Vec<Arc<Task>> {
        self.task_waiting.values()
            .map(|task| task.clone())
            .collect()
    }

    /// Run epoll_wait.
    pub fn wait(&mut self) {
        let capacity = 1024;
        let mut events = Vec::with_capacity(capacity);
        let timeout = 10;
        self.ready.clear();

        match syscall!(epoll_wait(self.epoll_fd, events.as_mut_ptr(),
                                  capacity as i32, timeout)) {
            Ok(num) => {
                unsafe {
                    events.set_len(num as usize);

                    for e in &events {
                        let fd  = e.u64 as RawFd;
                        self.ready.insert(fd, true);
                    }
                };
            }
            Err(_) => {
            }
        }
    }

    pub fn run(task: Arc<Task>) -> Poll<()> {
        let mut future_slot = task.future.lock().unwrap();
        if let Some(mut future) = future_slot.take() {
            let waker = waker_ref(&task);
            let context = &mut Context:: from_waker(&*waker);
            if let Poll::Pending = future.as_mut().poll(context) {
                *future_slot = Some(future);
                Poll::Pending
            } else {
//                *future_slot = Some(future);
                Poll::Ready(())
            }
        } else {
            panic!("No future in task!");
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
        let mut epoll = event_manager.fd_events();

        if let Some(flag) = epoll.ready.get(&self.fd) {
            if *flag {
                epoll.task_waiting.remove(&self.fd);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        } else {
            // Unlikely, though.
            Poll::Pending
        }
    }
}
