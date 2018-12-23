//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// BGP Master
//

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::cell::RefCell;

use super::super::core::master::ProtocolMaster;
//use super::super::core::message::master::ProtoToMaster;
//use super::super::core::message::master::MasterToProto;
//use super::super::core::message::zebra::ProtoToZebra;
use super::super::core::master::MasterInner;
//use super::super::core::protocols::ProtocolType;

pub struct BgpMaster {
    master: RefCell<Arc<ProtocolMaster>>
}

impl BgpMaster {
    pub fn new(master: Arc<ProtocolMaster>) -> BgpMaster {
        BgpMaster { master: RefCell::new(master) }
    }
}

impl MasterInner for BgpMaster {
    fn start(&self) {
//             _sender_p2m: mpsc::Sender<ProtoToMaster>,
//             _receiver_m2p: mpsc::Receiver<MasterToProto>,
//             _sender_p2z: mpsc::Sender<ProtoToZebra>) {

        loop {
            thread::sleep(Duration::from_secs(2));
//            sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Bgp, Duration::from_secs(8), 1)));
            println!("*** sender sending timer reg");
        }
    }
}

