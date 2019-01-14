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

use std::io;
use std::io::BufRead;
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
use mio::*;
use mio::unix::EventedFd;

//use super::event::*;
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
    timer_server: timer::Server,

    // channel to zebra
    // command API handler
}

impl RouterNexus {
    pub fn new() -> RouterNexus {
        RouterNexus {
            masters: HashMap::new(),
            timer_server: timer::Server::new()
        }
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, _receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let zebra = ZebraMaster { };
            zebra.start(sender_p2n, receiver_n2p);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_p2z)
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2n: mpsc::Sender<ProtoToNexus>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<NexusToProto>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();

        let handle = thread::spawn(move || {
            let protocol = Arc::new(ProtocolMaster::new(p));
            protocol.inner_set(
                match p {
                    ProtocolType::Ospf => Box::new(OspfMasterInner::new(protocol.clone())),
                    ProtocolType::Bgp => Box::new(BgpMaster::new(protocol.clone())),
                    _ => panic!("Not supported")
                });

            protocol.timers_set(timer::Client::new(protocol.clone()));
            protocol.start(sender_p2n, receiver_n2p, sender_p2z);
            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p)
    }

    //
    fn finish_protocol(&mut self, proto: &ProtocolType) {
        if let Some(tuple) = self.masters.remove(&proto) {
            tuple.sender.send(NexusToProto::ProtoTermination);

            match tuple.handle.join() {
                Ok(_ret) => {
                    debug!("protocol join succeeded");
                },
                Err(_err) => {
                    debug!("protocol join failed");
                }
            }
        }
    }

    //
    pub fn start(&mut self) {
        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();

        // Spawn zebra instance
        let (handle, sender, sender_p2z) =
            self.spawn_zebra(mpsc::Sender::clone(&sender_p2n));
        self.masters.insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        // MIO poll
        let poll = Poll::new().unwrap();

        let stdin = 0;
        let stdin_fd = EventedFd(&stdin);
        poll.register(&stdin_fd, Token(0), Ready::readable(), PollOpt::level()) .unwrap();
        
        'main: loop {
            // TBD: process CLI, API, poll()
            let mut events = Events::with_capacity(1024);
            poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();

            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        let mut line = String::new();
                        io::stdin().lock().read_line(&mut line).unwrap();

                        let command = line.trim();

                        match command {
                            "ospf" => {
                                // Spawn ospf instance
                                let (handle, sender) =
                                    self.spawn_protocol(ProtocolType::Ospf, mpsc::Sender::clone(&sender_p2n),
                                                        mpsc::Sender::clone(&sender_p2z));
                                self.masters.insert(ProtocolType::Ospf, MasterTuple { handle, sender });
                            },
                            "bgp" => {

                            },
                            "quit" => {
                                break 'main;
                            },
                            _ => {
                                println!("!Unknown command");
                            }
                        }
                    },
                    _ => {
                    }
                }
            }

            // process channels
//          for _m in &self.masters {
                while let Ok(d) = receiver.try_recv() {
                    match d {
                        ProtoToNexus::TimerRegistration((p, d, token)) => {
                            debug!("Received timer registration {} {}", p, token);

                            self.timer_server.register(p, d, token);
                        }
                        ProtoToNexus::ProtoException(s) => {
                            debug!("Received exception {}", s);
                        }
                    }
                }
//          }

            thread::sleep(Duration::from_millis(10));

            // Process timer
            match self.timer_server.pop_if_expired() {
                Some(entry) => {
                    match self.masters.get(&entry.protocol) {
                        Some(tuple) => {
                            let result =
                                tuple.sender.send(NexusToProto::TimerExpiration(entry.token));
                            // TODO
                            match result {
                                Ok(_ret) => {},
                                Err(_err) => {}
                            }
                        }
                        None => {
                            panic!("Unexpected error");
                        }
                    }
                },
                None => { }
            }
        }

        // Send termination message to all threads first.
        // TODO: is there better way to iterate hashmap and remove it at the same time?
        let mut v = Vec::new();
        for (proto, _tuple) in self.masters.iter_mut() {
            v.push(proto.clone());
        }

        for proto in &v {
            self.finish_protocol(proto);
        }

        // Nexus terminated.
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


