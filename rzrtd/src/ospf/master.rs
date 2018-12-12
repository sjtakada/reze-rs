//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// OSPF Master
//
//use super::master::ProtocolMaster;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use log::debug;

use super::super::core::message::master::ProtoToMaster;
use super::super::core::message::master::MasterToProto;
use super::super::core::message::zebra::ProtoToZebra;
use super::super::core::master::ProtocolMaster;
use super::super::core::protocols::ProtocolType;

pub struct OspfMaster {
}

impl ProtocolMaster for OspfMaster {
    fn start(&self,
             sender_p2m: mpsc::Sender<ProtoToMaster>,
             receiver_m2p: mpsc::Receiver<MasterToProto>,
             sender_p2z: mpsc::Sender<ProtoToZebra>) {

        sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Ospf, Duration::from_secs(10), 1)));
        debug!("OSPF Master: sender sending first timer reg");
        loop {
            while let Ok(d) = receiver_m2p.try_recv() {
                debug!("OSPF Master: received timer expiration");
                sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Ospf,
                                                                  Duration::from_secs(10), 1)));
                debug!("OSPF Master: sender sending another timer reg");
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

