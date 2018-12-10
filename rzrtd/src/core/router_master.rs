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

//use std::io;
//use std::io::prelude::*;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::marker::Send;

use super::protocols::ProtocolType;
use super::message::master::ProtoToMaster;
use super::message::master::MasterToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

trait ProtocolMaster {
    fn start(&self);
//    fn finish(&self);
}

pub struct ZebraMaster {
    // Zebra Message Receiver
//    receiver: Cell<mpsc::Receiver<ProtoToZebra>>
}

impl ProtocolMaster for ZebraMaster {
    fn start(&self) {
    }
}

pub struct OspfMaster {
}

impl ProtocolMaster for OspfMaster {
    fn start(&self) {
    }
}

pub struct BgpMaster {
}

impl ProtocolMaster for BgpMaster {
    fn start(&self) {
    }
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
        MasterFactory {}
    }

    pub fn get_protocol(&self, p: &ProtocolType) -> Arc<ProtocolMaster + Send + Sync> {
        match p {
            ProtocolType::Zebra => Arc::new(ZebraMaster{}),
            ProtocolType::Ospf => Arc::new(OspfMaster{}),
            ProtocolType::Bgp => Arc::new(BgpMaster{}),
            _ => panic!("Not supported")
        }
    }
}

pub struct RouterMaster {
    // Master Factory
    factory: MasterFactory,

    // ProtocolMaster map
    masters: HashMap<ProtocolType, MasterTuple>,

    // channel to zebra
    // timer facility
    // command API handler
}

impl RouterMaster {
    pub fn new() -> RouterMaster {
        RouterMaster {
            factory: MasterFactory::new(),
            masters: HashMap::new()
        }
    }

    // Construct ProtocolMaster instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2m: mpsc::Sender<ProtoToMaster>)
                   -> (JoinHandle<()>, mpsc::Sender<MasterToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterMaster to ProtocolMaster
        let (sender, receiver) = mpsc::channel::<MasterToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let proto = self.factory.get_protocol(&ProtocolType::Zebra);
        let handle = thread::spawn(move || {
            loop {
                proto.start();

                // handle receiver chan

                thread::sleep(Duration::from_secs(2));
                sender_p2m.send(ProtoToMaster::TimerRegistration((1, 2)));
                println!("*** sender sending timer reg");
            }

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender, sender_p2z)
    }

    // Construct ProtocolMaster instance and spawn a thread.
    fn spawn_master(&self, p: ProtocolType,
                    sender_p2m: mpsc::Sender<ProtoToMaster>,
                    sender_p2z: mpsc::Sender<ProtoToZebra>)
                    -> (JoinHandle<()>, mpsc::Sender<MasterToProto>) {
        // Create channel from RouterMaster to ProtocolMaster
        let (sender, receiver) = mpsc::channel::<MasterToProto>();
        let proto = self.factory.get_protocol(&p);
        let handle = thread::spawn(move || {
            loop {
                proto.start();

                // handle receiver chan

                thread::sleep(Duration::from_secs(2));
                sender_p2m.send(ProtoToMaster::TimerRegistration((1, 2)));
                println!("*** sender sending timer reg");
            }

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender)
    }

    pub fn start(&mut self) {
        // Create multi sender channel from ProtocolMaster to RouterMaster
        let (sender_p2m, receiver) = mpsc::channel::<ProtoToMaster>();

        // Spawn zebra instance
        let (handle, sender, sender_p2z) =
            self.spawn_zebra(mpsc::Sender::clone(&sender_p2m));
        self.masters.insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        // Spawn ospf instance
        let (handle, sender) =
            self.spawn_master(ProtocolType::Ospf, mpsc::Sender::clone(&sender_p2m),
                              mpsc::Sender::clone(&sender_p2z));
        self.masters.insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        loop {
            // process CLI, API

            println!("*** loop 1");
            // process channel
            while let Ok(d) = receiver.try_recv() {
                match d {
                    ProtoToMaster::TimerRegistration((x, y)) => {
                        println!("*** receive timer reg {} {}", x, y);
                    }
                    ProtoToMaster::ProtoTermination(i) => {
                    }
                }
            }

            println!("*** loop 3");
            thread::sleep(Duration::from_secs(1));

            // process timer

            // push expired event to channel
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

