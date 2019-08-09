//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra Master
//

use log::debug;

use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

//use crate::core::event::*;

use crate::core::protocols::ProtocolType;
use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;
use crate::core::message::zebra::ProtoToZebra;
use crate::core::message::zebra::ZebraToProto;

use super::netlink;

// Store Zebra Client related information.
struct ClientTuple {
    // Channel sender from Zebra to Protocol
    sender: mpsc::Sender<ZebraToProto>,
}

// Zebra Master.
pub struct ZebraMaster {


    //
    clients: HashMap<ProtocolType, ClientTuple>
}

impl ZebraMaster {
    pub fn new() -> ZebraMaster {
        ZebraMaster { clients: HashMap::new() }
    }

    pub fn start(&mut self,
                 _sender_p2n: mpsc::Sender<ProtoToNexus>,
                 receiver_n2p: mpsc::Receiver<NexusToProto>,
                 receiver_p2z: mpsc::Receiver<ProtoToZebra>) {

        // Init netlink socket.



        // Main loop for zebra
        'main: loop {
            // XXX: handle receiver_p2z 
            while let Ok(d) = receiver_p2z.try_recv() {
                match d {
                    ProtoToZebra::RegisterProto((proto, sender_z2p)) => {
                        self.clients.insert(proto, ClientTuple { sender: sender_z2p });
                        debug!("Register Protocol {}", proto);
                    },
                    ProtoToZebra::RouteAdd(_i) => {
                    },
                    ProtoToZebra::RouteLookup(_i) => {
                    },
                }
            }

            // XXX: handle receiver_n2p
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

                thread::sleep(Duration::from_millis(100));
            }

            // TODO: Some cleanup has to be done for inner.
            // inner.finish();
        }
        debug!("Zebra terminated");
    }
}

