//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Zebra Master
//

//use std::thread;
//use std::time::Duration;
use std::sync::mpsc;

use super::super::core::message::master::ProtoToMaster;
use super::super::core::message::master::MasterToProto;

pub struct ZebraMaster {
}

impl ZebraMaster {
    pub fn start(&self,
                 sender_p2m: mpsc::Sender<ProtoToMaster>,
                 receiver_m2p: mpsc::Receiver<MasterToProto>) {
        // Main loop for zebra
        loop {
            // handle receiver chan
//            thread::sleep(Duration::from_secs(2));
//            sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Zebra, Duration::from_secs(5), 1)));
//            println!("*** sender sending timer reg");

            // TBD: TimerClient
        }
    }
}

