//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// Router Nexus
//   Global container.
//   Initiate routing threads.
//   Handle messages from controller.
//   Dispatch commands to each protocol.
//   Run timer server and notify clients.
//

use log::debug;

use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
use std::boxed::Box;
//use std::cell::Cell;
//use std::cell::RefCell;
use std::time::Duration;
//use std::time::Instant;

use super::event::*;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;

use super::master::ProtocolMaster;
use crate::zebra::master::ZebraMaster;
use crate::bgp::master::BgpMaster;
use crate::ospf::master::OspfMasterInner;

use super::timer;

struct MasterTuple {
    // Thread Join handle
    handle: JoinHandle<()>,

    // Channel sender from Master To Protocol
    sender: mpsc::Sender<NexusToProto>,
}


pub struct RouterNexus {
    // MasterInner map
    masters: HashMap<ProtocolType, MasterTuple>,

    // Timer server
    timer: timer::Server,

    // channel to zebra
    // command API handler
}

impl RouterNexus {
    pub fn new() -> RouterNexus {
        RouterNexus {
            masters: HashMap::new(),
            timer: timer::Server::new()
        }
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2m: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_m2p, receiver_m2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, _receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let zebra = ZebraMaster { };
            zebra.start(sender_p2m, receiver_m2p);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_m2p, sender_p2z)
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2m: mpsc::Sender<ProtoToNexus>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<NexusToProto>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_m2p, receiver_m2p) = mpsc::channel::<NexusToProto>();

        let handle = thread::spawn(move || {
            let protocol = Arc::new(ProtocolMaster::new(p));
            protocol.inner_set(
                match p {
                    ProtocolType::Ospf => Box::new(OspfMasterInner::new(protocol.clone())),
                    ProtocolType::Bgp => Box::new(BgpMaster::new(protocol.clone())),
                    _ => panic!("Not supported")
                });

            protocol.timers_set(timer::Client::new(protocol.clone()));
            protocol.start(sender_p2m, receiver_m2p, sender_p2z);
            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_m2p)
    }

    pub fn start(&mut self) {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2m, receiver) = mpsc::channel::<ProtoToNexus>();

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
            for _m in &self.masters {
                while let Ok(d) = receiver.try_recv() {
                    match d {
                        ProtoToNexus::TimerRegistration((p, d, token)) => {
                            debug!("ProtoToNexus receive timer reg {} {}", p, token);

                            self.timer.register(p, d, token);
                        }
                        ProtoToNexus::ProtoTermination(_i) => {
                            debug!("ProtoToNexus TBD");
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
                            let result =
                                m.sender.send(NexusToProto::TimerExpiration(entry.token));
                            // TODO
                            match result {
                                Ok(_ret) => {},
                                Err(_err) => {}
                            }
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


