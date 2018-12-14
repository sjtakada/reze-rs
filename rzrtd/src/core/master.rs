//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018 Toshiaki Takada
//
// Router Master
//   Global container.
//   Initiate routing threads.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use log::debug;

//use std::io;
//use std::io::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
//use std::rc::Rc;
use std::boxed::Box;
use std::time::Duration;
//use std::time::Instant;
//use std::marker::Send;

use super::protocols::ProtocolType;
use super::message::master::ProtoToMaster;
use super::message::master::MasterToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

use super::timer;

use super::super::zebra::master::ZebraMaster;
use super::super::bgp::master::BgpMaster;
use super::super::ospf::master::OspfMaster;

pub struct ProtocolMaster {
    inner: Box<MasterInner + Send + Sync>,
}

impl ProtocolMaster {
    pub fn start(&self,
             sender_p2m: mpsc::Sender<ProtoToMaster>,
             receiver_m2p: mpsc::Receiver<MasterToProto>,
             sender_p2z: mpsc::Sender<ProtoToZebra>) {
        self.inner.start(sender_p2m, receiver_m2p, sender_p2z);
    }
}

pub trait MasterInner {
    fn start(&self,
             sender_p2m: mpsc::Sender<ProtoToMaster>,
             receiver_m2p: mpsc::Receiver<MasterToProto>,
             sender_p2z: mpsc::Sender<ProtoToZebra>);
//    fn finish(&self);
}

struct MasterTuple {
    // Thread Join handle
    handle: JoinHandle<()>,

    // Channel sender from Master To Protocol
    sender: mpsc::Sender<MasterToProto>,
}

// Master Factory
//   Each master runs its own thread, and contains all data to run protocol.
//   Each master may communicate through several channels.
struct MasterFactory;

impl MasterFactory {
    pub fn new() -> MasterFactory {
        MasterFactory { }
    }

    pub fn get_zebra(&self) -> ZebraMaster {
        ZebraMaster { }
    }

    pub fn get_protocol(&self, p: &ProtocolType) -> ProtocolMaster {
        ProtocolMaster {
            inner: match p {
                ProtocolType::Ospf => Box::new(OspfMaster { }),
                ProtocolType::Bgp => Box::new(BgpMaster { }),
                _ => panic!("Not supported")
            }
        }
    }
}

pub struct RouterMaster {
    // Master Factory
    factory: MasterFactory,

    // MasterInner map
    masters: HashMap<ProtocolType, MasterTuple>,

    // Timer server
    timer: timer::Server,

    // channel to zebra
    // command API handler
}

impl RouterMaster {
    pub fn new() -> RouterMaster {
        RouterMaster {
            factory: MasterFactory::new(),
            masters: HashMap::new(),
            timer: timer::Server::new()
        }
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2m: mpsc::Sender<ProtoToMaster>)
                   -> (JoinHandle<()>, mpsc::Sender<MasterToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterMaster to MasterInner
        let (sender_m2p, receiver_m2p) = mpsc::channel::<MasterToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let zebra = self.factory.get_zebra();
        let handle = thread::spawn(move || {
            zebra.start(sender_p2m, receiver_m2p);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_m2p, sender_p2z)
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2m: mpsc::Sender<ProtoToMaster>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<MasterToProto>) {
        // Create channel from RouterMaster to MasterInner
        let (sender_m2p, receiver_m2p) = mpsc::channel::<MasterToProto>();
        let protocol = self.factory.get_protocol(&p);
        let handle = thread::spawn(move || {
            protocol.start(sender_p2m, receiver_m2p, sender_p2z);
            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_m2p)
    }

    pub fn start(&mut self) {
        // Create multi sender channel from MasterInner to RouterMaster
        let (sender_p2m, receiver) = mpsc::channel::<ProtoToMaster>();

        // Spawn zebra instance
        let (handle, sender, sender_p2z) =
            self.spawn_zebra(mpsc::Sender::clone(&sender_p2m));
        self.masters.insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        // Spawn ospf instance
        let (handle, sender) =
            self.spawn_protocol(ProtocolType::Ospf, mpsc::Sender::clone(&sender_p2m),
                                mpsc::Sender::clone(&sender_p2z));
        self.masters.insert(ProtocolType::Ospf, MasterTuple { handle, sender });

        loop {
            // TBD: process CLI, API, poll()

            // process channels
            for m in &self.masters {
                while let Ok(d) = receiver.try_recv() {
                    match d {
                        ProtoToMaster::TimerRegistration((p, d, token)) => {
                            debug!("ProtoToMaster receive timer reg {} {}", p, token);

                            self.timer.register(p, d, token);
                        }
                        ProtoToMaster::ProtoTermination(i) => {
                            debug!("ProtoToMaster TBD");
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));

            // process timer
            match self.timer.pop_if_expired() {
                Some(entry) => {
                    match self.masters.get(&entry.protocol) {
                        Some(m) => {
                            m.sender.send(MasterToProto::TimerExpiration(entry.token));
                        }
                        None => {
                            panic!("error");
                        }
                    }
                },
                _ => { }
            }
        }
    }
}

/*
struct BootStrapTask {

}

impl Future for BootStrapTask {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut line).unwrap();

            match line.as_ref() {
                "ospf\n" => {
                    println!("% start ospf {}", line);
                },
                "end\n" => {
                    println!("% end");
                    break;
                }
                _ => {
                    println!("% Unknown command");
                }
            }
        }

        Ok(Async::Ready(()))
    }
}
*/


