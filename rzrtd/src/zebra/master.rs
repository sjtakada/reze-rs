//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra Master
//

use log::{debug, error};

use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use crate::core::event::*;

use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;

pub struct ZebraMaster {
}

impl ZebraMaster {
    pub fn start(&self,
                 _sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>) {

        // XXX: handle receiver_p2z 

        // Main loop for zebra
        'main: loop {
            // Take care of protocol specific stuff.
            // inner.start();

            // 
            loop {
                while let Ok(d) = receiver_n2p.try_recv() {
                    match d {
                        NexusToProto::TimerExpiration(token) => {
                            debug!("Received TimerExpiration with token {}", token);

                            /*
                            match self.timer_handler_get(token) {
                                Some(handler) => {
                                    handler.handle(EventType::TimerEvent);
                                },
                                None => {
                                    error!("Handler doesn't exist");
                                }
                            }
                             */
                        },
                        NexusToProto::PostConfig((command, _v)) => {
                            debug!("Received PostConfig with command {}", command);
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
}

