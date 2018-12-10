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
//use std::marker::Send;

use super::protocols::ProtocolType;
use super::message::master::ProtoToMaster;
use super::message::master::MasterToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

use super::super::zebra::master::ZebraMaster;
use super::super::bgp::master::BgpMaster;
use super::super::ospf::master::OspfMaster;

pub trait ProtocolMaster {
    fn start(&self,
             sender_p2m: mpsc::Sender<ProtoToMaster>,
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
        MasterFactory {}
    }

    pub fn get_zebra(&self) -> Arc<ZebraMaster> {
        Arc::new(ZebraMaster{})
    }

    pub fn get_protocol(&self, p: &ProtocolType) -> Arc<ProtocolMaster + Send + Sync> {
        match p {
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
        let zebra = self.factory.get_zebra();
        let handle = thread::spawn(move || {
            zebra.start(sender_p2m);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender, sender_p2z)
    }

    // Construct ProtocolMaster instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2m: mpsc::Sender<ProtoToMaster>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<MasterToProto>) {
        // Create channel from RouterMaster to ProtocolMaster
        let (sender, receiver) = mpsc::channel::<MasterToProto>();
        let protocol = self.factory.get_protocol(&p);
        let handle = thread::spawn(move || {
            protocol.start(sender_p2m, sender_p2z);
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
            self.spawn_protocol(ProtocolType::Ospf, mpsc::Sender::clone(&sender_p2m),
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


