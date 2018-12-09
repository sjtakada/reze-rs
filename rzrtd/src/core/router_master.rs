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

use std::io;
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;
use std::marker::Send;

use super::master_message::MessageToMaster;
use super::master_message::MessageToProto;
use super::protocols::ProtocolType;

trait ProtocolMaster {
    fn start(&self);
//    fn finish(&self);
}

pub struct ZebraMaster {
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

/*
impl Protocol {
    pub fn get(p: &ProtocolType) -> Protocol {
        match p {
            ProtocolType::Zebra => Protocol::Zebra(Zebra {}),
            ProtocolType::Ospf => Protocol::Ospf(Ospf {}),
            ProtocolType::Bgp => Protocol::Bgp(Bgp {}),
            _ => panic!("Not supported")
        }
    }
}
 */

pub struct RouterMaster {
    // Master Factory
    factory: MasterFactory,

    // channel to zebra
    // timer facility
    // command API handler
}

impl RouterMaster {
    pub fn new() -> RouterMaster {
        let factory = MasterFactory::new();

        RouterMaster { factory }
    }

    // set returned sender channel with protocol id to map
    pub fn proto_init(&self, p: &ProtocolType,
                      sender_p2m: mpsc::Sender<MessageToMaster>)
                      -> mpsc::Sender<MessageToProto> {
        let (sender, receiver) = mpsc::channel::<MessageToProto>();
        let proto = self.factory.get_protocol(&p);
        let proto_t = thread::spawn(move || {
            loop {
                proto.start();

                // handle receiver chan

                thread::sleep(Duration::from_secs(2));
                sender_p2m.send(MessageToMaster::TimerRegistration((1, 2)));
                println!("*** sender sending timer reg");
            }
        });

        sender
    }

    pub fn run(&mut self) {
        // Create multi sender channel.
        let (sender_proto, receiver) = mpsc::channel::<MessageToMaster>();

        // Spawn zebra instance
        self.proto_init(&ProtocolType::Zebra, mpsc::Sender::clone(&sender_proto));

        // Spawn ospf instance
        self.proto_init(&ProtocolType::Ospf, mpsc::Sender::clone(&sender_proto));

        /*
        let (chan_tx_m2p, chan_rx_m2p) = mpsc::channel::<MessageToProto>();
        let sender = mpsc::Sender::clone(&sender_proto);
        let zebra = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(2));
                sender.send(MessageToMaster::TimerRegistration((1, 2)));
                println!("*** sender sending timer reg");
            }
        });
        
        // spawn ospf instance
        let (chan_tx_m2p, chan_rx_m2p) = mpsc::channel::<MessageToProto>();
        let sender = mpsc::Sender::clone(&sender_proto);
        let ospf = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(3));
                sender.send(MessageToMaster::TimerRegistration((3, 4)));
                println!("*** sender sending timer reg");
            }
        });
         */

        loop {
            // process CLI, API

            println!("*** loop 1");
            // process channel
            while let Ok(d) = receiver.try_recv() {
                match d {
                    MessageToMaster::TimerRegistration((x, y)) => {
                        println!("*** receive timer reg {} {}", x, y);
                    }
                    MessageToMaster::ProtoTermination(i) => {
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

