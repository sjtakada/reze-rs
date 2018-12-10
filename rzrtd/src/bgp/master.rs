//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// BGP Master
//
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use super::super::core::message::master::ProtoToMaster;
use super::super::core::message::zebra::ProtoToZebra;
use super::super::core::master::ProtocolMaster;

pub struct BgpMaster {
}

impl ProtocolMaster for BgpMaster {
    fn start(&self,
             sender_p2m: mpsc::Sender<ProtoToMaster>,
             sender_p2z: mpsc::Sender<ProtoToZebra>) {
        loop {
            thread::sleep(Duration::from_secs(2));
            sender_p2m.send(ProtoToMaster::TimerRegistration((1, 2)));
            println!("*** sender sending timer reg");
        }
    }
}

