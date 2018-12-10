//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Zebra Master
//
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use super::super::core::message::master::ProtoToMaster;

pub struct ZebraMaster {
    // Zebra Message Receiver
//    receiver: Cell<mpsc::Receiver<ProtoToZebra>>
}

impl ZebraMaster {
    pub fn start(&self, sender_p2m: mpsc::Sender<ProtoToMaster>) {
        // Main loop for zebra
        loop {
            // handle receiver chan
            thread::sleep(Duration::from_secs(2));
            sender_p2m.send(ProtoToMaster::TimerRegistration((1, 2)));
            println!("*** sender sending timer reg");
        }
    }
}

