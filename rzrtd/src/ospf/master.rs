//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// OSPF Master
//

//use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::cell::RefCell;

use log::debug;

use super::super::core::event::*;

use super::super::core::master::ProtocolMaster;
//use super::super::core::message::master::ProtoToMaster;
//use super::super::core::message::master::MasterToProto;
//use super::super::core::message::zebra::ProtoToZebra;
use super::super::core::master::MasterInner;
use super::super::core::protocols::ProtocolType;

pub struct OspfMaster {
    // TODO: ??? could it be just reference ???
    master: RefCell<Arc<ProtocolMaster>>
}

impl OspfMaster {
    pub fn new(master: Arc<ProtocolMaster>) -> OspfMaster {
        OspfMaster { master: RefCell::new(master) }
    }
}

impl EventHandler for OspfMaster {
    fn handle(&self, e: EventType) {
        match e {
            _ => { println!("*** handle_event") }
        }
    }
}

impl MasterInner for OspfMaster {
    fn start(&self) {
//             sender_p2m: mpsc::Sender<ProtoToMaster>,
//             receiver_m2p: mpsc::Receiver<MasterToProto>,
//             _sender_p2z: mpsc::Sender<ProtoToZebra>) {

        let master = self.master.borrow();
        master.timer_register(ProtocolType::Ospf, Duration::from_secs(10), self);

//        let result =
//            sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Ospf, Duration::from_secs(10), 1)));
        // TODO
//        match result {
//            Ok(_ret) => {},
//            Err(_err) => {}
//        }
        
        debug!("OSPF Master: sender sending first timer reg");
/*
        loop {
            while let Ok(_d) = receiver_m2p.try_recv() {
                debug!("OSPF Master: received timer expiration");
                let result =
                    sender_p2m.send(ProtoToMaster::TimerRegistration((ProtocolType::Ospf,
                                                                      Duration::from_secs(10), 1)));
                // TODO
                match result {
                    Ok(_ret) => {},
                    Err(_err) => {}
                }

                debug!("OSPF Master: sender sending another timer reg");
            }
        }
*/
    }
}

