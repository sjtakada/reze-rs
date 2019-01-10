//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Protocol Master
//   Global container.
//   Initiate routing threads.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use log::debug;

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
//use std::rc::Rc;
use std::boxed::Box;
//use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;
//use std::time::Instant;
//use std::marker::Send;
use std::sync::Mutex;

use super::event::*;

use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
//use super::message::zebra::ZebraToProto;

use super::timer;

pub struct ProtocolMaster {
    // Protocol specific inner
    inner: RefCell<Option<Box<MasterInner>>>,

    // Timer
    timers: RefCell<Option<timer::Client>>,

    // Sender channel for ProtoToNexus Message
    sender_p2m: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    // Sender channel for ProtoToZebra Message
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,

    // TODO: integrate this into timers, probably
    lock: Mutex<i32>,
}

impl ProtocolMaster {
    pub fn new(_p: ProtocolType) -> ProtocolMaster {
        ProtocolMaster {
            inner: RefCell::new(None),
            timers: RefCell::new(None),
            sender_p2m: RefCell::new(None),
            sender_p2z: RefCell::new(None),
            lock: Mutex::new(0),
        }
    }

    pub fn timer_handler_get(&self, token: u32) -> Option<Arc<EventHandler + Send + Sync>> {
//        let lock = self.lock.lock().unwrap();
        let mut some_handler = None;
        if let Some(ref mut timers) = *self.timers.borrow_mut() {
            some_handler = timers.unregister(token);
        }
        some_handler
    }

    pub fn start(&self,
                 sender_p2m: mpsc::Sender<ProtoToNexus>,
                 receiver_m2p: mpsc::Receiver<NexusToProto>,
                 sender_p2z: mpsc::Sender<ProtoToZebra>) {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            self.sender_p2m.borrow_mut().replace(sender_p2m);
            self.sender_p2z.borrow_mut().replace(sender_p2z);

            inner.start();

            loop {
                while let Ok(d) = receiver_m2p.try_recv() {
                    match d {
                        NexusToProto::TimerExpiration(token) => {
                            debug!("Received TimerExpiration with token {}", token);

                            match self.timer_handler_get(token) {
                                Some(handler) => {
                                    handler.handle(EventType::TimerEvent);
                                },
                                None => {
                                    debug!("Handler doesn't exist");
                                }
                            }
                        },
                        _ => {
                            debug!("Not implemented");
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // TODO: may return value
    pub fn timer_register(&self, p: ProtocolType, d: Duration, handler: Arc<EventHandler + Send + Sync>) {
        if let Some(ref mut sender) = *self.sender_p2m.borrow_mut() {
//            let lock = self.lock.lock().unwrap();
            if let Some(ref mut timers) = *self.timers.borrow_mut() {
                let token = timers.register(handler, d);
                let result = sender.send(ProtoToNexus::TimerRegistration((p, d, token)));

                debug!("Timer registration with token {}", token);

                match result {
                    // TODO
                    Ok(_ret) => { println!("Ok") },
                    Err(err) => { println!("Err {}", err) }
                }
            }
        }
    }

    pub fn inner_set(&self, inner: Box<MasterInner>) {
        self.inner.borrow_mut().replace(inner);
    }

    pub fn timers_set(&self, timers: timer::Client) {
        self.timers.borrow_mut().replace(timers);
    }
}

pub trait MasterInner {
    fn start(&self);
//             sender_p2m: mpsc::Sender<ProtoToNexus>,
//             receiver_m2p: mpsc::Receiver<NexusToProto>,
//             sender_p2z: mpsc::Sender<ProtoToZebra>);

//    fn finish(&self);
}

