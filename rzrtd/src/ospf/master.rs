//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// OSPF Master
//

//use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::cell::Cell;
use std::cell::RefCell;
use std::marker::Send;
use std::marker::Sync;

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
    master: RefCell<Arc<ProtocolMaster>>,

    // OSPF instance vector.
    ospf: RefCell<Vec<Arc<Ospf>>>,
}

impl OspfMaster {
    pub fn new(master: Arc<ProtocolMaster>) -> OspfMaster {
        OspfMaster { master: RefCell::new(master),
                     ospf: RefCell::new(Vec::new()) }
    }
}

impl MasterInner for OspfMaster {
    fn start(&self) {
        let master = self.master.borrow();
        let ospf = Arc::new(Ospf { });
        self.ospf.borrow_mut().push(ospf.clone());

        master.timer_register(ProtocolType::Ospf, Duration::from_secs(10), ospf.clone());

        debug!("OSPF Master: sender sending first timer reg");
    }
}


//unsafe impl Send for Ospf {}
//unsafe impl Sync for Ospf {}

struct Ospf {

}

impl EventHandler for Ospf {
    fn handle(&self, e: EventType) {
        match e {
            // XXX: timer expire, send another hello
            EventType::TimerEvent => {
                println!("*** receive tiemr expiration");
            }
            _ => { println!("*** handle_event") }
        }
    }
}

