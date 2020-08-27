//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// OSPF Master
//

use log::debug;

//use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Weak;
use std::cell::RefCell;

use eventum::core::*;

use common::error::*;

use crate::core::master::ProtocolMaster;
use crate::core::master::MasterInner;
use crate::core::protocols::ProtocolType;

pub struct OspfMasterInner {
    // TODO: ??? could it be just reference ???
    master: RefCell<Arc<ProtocolMaster>>,

    // OSPF instance vector.
    ospf: RefCell<Vec<Arc<Ospf>>>,
}

impl OspfMasterInner {
    pub fn new(master: Arc<ProtocolMaster>) -> OspfMasterInner {
        OspfMasterInner { master: RefCell::new(master),
                          ospf: RefCell::new(Vec::new()) }
    }
}

impl MasterInner for OspfMasterInner {
    fn start(&self) {
        // Create OSPF instance and Inner and reference each other
        let master = self.master.borrow();
        let ospf = Arc::new(Ospf::new(master.clone()));
        let inner = OspfInner{ ospf: Arc::downgrade(&ospf) };
        ospf.inner.replace(Some(inner));

        // Set OSPF instance to vector
        self.ospf.borrow_mut().push(ospf.clone());

        // Start OSPF hello timer
        ospf.start();

        debug!("sent first timer reg");
    }
}

struct OspfHelloTimer {
    ospf: Weak<Ospf>
}

unsafe impl Send for OspfHelloTimer {}
unsafe impl Sync for OspfHelloTimer {}

struct OspfInner {
    ospf: Weak<Ospf>
}

impl OspfInner {
    // Start Hello Timer
    pub fn start(&self) {
        let timer = OspfHelloTimer { ospf: self.ospf.clone() };
        if let Some(ospf) = self.ospf.upgrade() {
            let master = ospf.master.borrow();
            master.timer_register(ProtocolType::Ospf, Duration::from_secs(10), Arc::new(timer));
        }
    }
}

struct Ospf {
    master: RefCell<Arc<ProtocolMaster>>,

    inner: RefCell<Option<OspfInner>>
}

impl Ospf {
    pub fn new(master: Arc<ProtocolMaster>) -> Ospf {
        Ospf {
            master: RefCell::new(master),
            inner: RefCell::new(None),
        }
    }

    pub fn start(&self) {
        if let Some(ref mut inner) = *self.inner.borrow_mut() {
            inner.start();
        }
    }
}

impl EventHandler for OspfHelloTimer {
    fn handle(&self, e: EventType) -> Result<(), EventError> {
        match e {
            EventType::TimerEvent => {
                debug!("Hello Timer fired");
                if let Some(ospf) = self.ospf.upgrade() {
                    debug!("Restart Hello Timer");
                    ospf.start();
                }
            }
            _ => {
                debug!("Unknown event");
            }
        }

        Ok(())
    }
}

