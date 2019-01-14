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

use log::{debug, error};

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

    // Sender channel for ProtoToNexus Message
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    // Sender channel for ProtoToZebra Message
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,

    // Timer Client
    timer_client: RefCell<Option<timer::Client>>,
}

impl ProtocolMaster {
    pub fn new(_p: ProtocolType) -> ProtocolMaster {
        ProtocolMaster {
            inner: RefCell::new(None),
            timer_client: RefCell::new(None),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    fn timer_handler_get(&self, token: u32) -> Option<Arc<EventHandler + Send + Sync>> {
        let mut some_handler = None;
        if let Some(ref mut timer_client) = *self.timer_client.borrow_mut() {
            some_handler = timer_client.unregister(token);
        }
        some_handler
    }

    pub fn start(&self,
                 sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>,
                 sender_p2z: mpsc::Sender<ProtoToZebra>) {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            self.sender_p2n.borrow_mut().replace(sender_p2n);
            self.sender_p2z.borrow_mut().replace(sender_p2z);

            // Take care of protocol specific stuff.
            inner.start();

            // 
            loop {
                while let Ok(d) = receiver_n2p.try_recv() {
                    match d {
                        NexusToProto::TimerExpiration(token) => {
                            debug!("Received TimerExpiration with token {}", token);

                            match self.timer_handler_get(token) {
                                Some(handler) => {
                                    handler.handle(EventType::TimerEvent);
                                },
                                None => {
                                    error!("Handler doesn't exist");
                                }
                            }
                        },
                        NexusToProto::PostConfig((command, _v)) => {
                            debug!("Received PostConfig with command {}", command);
                        },
                        NexusToProto::ProtoTermination => {
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // TODO: may return value
    pub fn timer_register(&self, p: ProtocolType, d: Duration, handler: Arc<EventHandler + Send + Sync>) {
        if let Some(ref mut sender) = *self.sender_p2n.borrow_mut() {
            if let Some(ref mut timer_client) = *self.timer_client.borrow_mut() {
                let token = timer_client.register(handler, d);
                let result = sender.send(ProtoToNexus::TimerRegistration((p, d, token)));

                match result {
                    // TODO
                    Ok(_ret) => {
                        debug!("Sent Timer Registration with token {}", token);
                    },
                    Err(err) => {
                        debug!("Error sending Timer Registration with token {}: error {}", token, err)
                    }
                }
            }
        }
    }

    pub fn inner_set(&self, inner: Box<MasterInner>) {
        self.inner.borrow_mut().replace(inner);
    }

    pub fn timers_set(&self, timers: timer::Client) {
        self.timer_client.borrow_mut().replace(timers);
    }
}

pub trait MasterInner {
    fn start(&self);
//    fn finish(&self);
}

