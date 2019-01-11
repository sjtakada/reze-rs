//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Zebra Master
//

//use std::thread;
//use std::time::Duration;
use std::sync::mpsc;

use crate::core::message::nexus::ProtoToNexus;
use crate::core::message::nexus::NexusToProto;

pub struct ZebraMaster {
}

impl ZebraMaster {
    pub fn start(&self,
                 _sender_p2n: mpsc::Sender<ProtoToNexus>,
                 _receiver_n2p: mpsc::Receiver<NexusToProto>) {
        // Main loop for zebra
        loop {
            // handle receiver chan
//            thread::sleep(Duration::from_secs(2));
//            sender_p2n.send(ProtoToMaster::TimerRegistration((ProtocolType::Zebra, Duration::from_secs(5), 1)));
//            println!("*** sender sending timer reg");

            // TBD: TimerClient
        }
    }
}

