//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Protocol Master
//   Global container.
//   Initiate routing threads.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::boxed::Box;
use std::cell::RefCell;
use std::time::Duration;

use log::{debug, error};

use eventum::core::*;

use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

use super::timer;

/// ProtocolMaster.
pub struct ProtocolMaster {

    /// Protocol specific inner.
    inner: RefCell<Option<Box<dyn MasterInner>>>,

    /// Sender channel for ProtoToNexus Message.
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    /// Sender channel for ProtoToZebra Message.
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,

    /// Timer Client.
    timer_client: RefCell<Option<timer::Client>>,
}

/// ProtocolMaster implementation.
impl ProtocolMaster {

    /// Constructor.
    pub fn new(_p: ProtocolType) -> ProtocolMaster {
        ProtocolMaster {
            inner: RefCell::new(None),
            timer_client: RefCell::new(None),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    /// TBD
    fn timer_handler_get(&self, token: u32) -> Option<Arc<dyn EventHandler + Send + Sync>> {
        let mut some_handler = None;
        if let Some(ref mut timer_client) = *self.timer_client.borrow_mut() {
            some_handler = timer_client.unregister(token);
        }
        some_handler
    }

    /// Entry point of protocol master.
    pub fn start(&self,
                 sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>,
                 sender_p2z: mpsc::Sender<ProtoToZebra>,
                 _receiver_z2p: mpsc::Receiver<ZebraToProto>) {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            self.sender_p2n.borrow_mut().replace(sender_p2n);
            self.sender_p2z.borrow_mut().replace(sender_p2z);

            // Take care of protocol specific stuff.
            inner.start();

            // TODO take care of receiver_z2p.try_recv()

            // 
            'main: loop {
                while let Ok(d) = receiver_n2p.try_recv() {
                    match d {
                        NexusToProto::TimerExpiration(token) => {
                            debug!("Received TimerExpiration with token {}", token);

                            match self.timer_handler_get(token) {
                                Some(handler) => {
                                    if let Err(err) = handler.handle(EventType::TimerEvent) {
                                        error!("Timer handler error {}", err);
                                    }
                                },
                                None => {
                                    error!("Handler doesn't exist");
                                }
                            }
                        },
                        NexusToProto::ConfigRequest((index, method, path, body)) => {
                            debug!("Received ConfigRequest with command {} {} {} {:?}", index, method, path, body);
                        },
                        NexusToProto::ExecRequest((index, method, path, body)) => {
                            debug!("Received ConfigRequest with command {} {} {} {:?}", index, method, path, body);
                        },
                        NexusToProto::ProtoTermination => {
                            debug!("Received ProtoTermination");
                            break 'main;
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }

            // TODO: Some cleanup has to be done for inner.
            // inner.finish();

            debug!("Protocol terminated");
        }
    }

    // TODO: may return value
    pub fn timer_register(&self, p: ProtocolType, d: Duration, handler: Arc<dyn EventHandler + Send + Sync>) {
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

    /// Set inner to master.
    pub fn inner_set(&self, inner: Box<dyn MasterInner>) {
        self.inner.borrow_mut().replace(inner);
    }

    /// Set timers to master.
    pub fn timers_set(&self, timers: timer::Client) {
        self.timer_client.borrow_mut().replace(timers);
    }
}

/// MasterInner trait.
pub trait MasterInner {
    fn start(&self);
//    fn finish(&self);
}

