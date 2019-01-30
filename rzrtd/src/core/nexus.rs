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
use log::error;

use std::io;
use std::io::BufRead;
use std::io::Read;
use std::env;
//use std::fs::File;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
use std::boxed::Box;
//use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;
//use std::time::Instant;
//use std::path::PathBuf;

use quick_error::*;
use mio::*;
use mio::unix::EventedFd;
use mio_uds::UnixListener;
use mio_uds::UnixStream;

//use std::os::unix::io::IntoRawFd;

//use super::event::*;
use super::protocols::ProtocolType;
use super::message::nexus::ProtoToNexus;
use super::message::nexus::NexusToProto;
use super::message::zebra::ProtoToZebra;
use super::message::zebra::ZebraToProto;

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

    // Sender channel for ProtoToNexus
    sender_p2n: RefCell<Option<mpsc::Sender<ProtoToNexus>>>,

    // Sender channel for ProtoToZebra
    sender_p2z: RefCell<Option<mpsc::Sender<ProtoToZebra>>>,
}

quick_error! {
    #[derive(Debug)]
    enum CoreError {
        NexusTermination {
            description("Nexus is terminated")
            display(r#"Nexus is terminated"#)
        }
        CommandNotFound(s: String) {
            description("The command could not be found")
            display(r#"The command "{}" could not be found"#, s)
        }
    }
}

impl RouterNexus {
    pub fn new() -> RouterNexus {
        RouterNexus {
            masters: HashMap::new(),
            timer_server: timer::Server::new(),
            sender_p2n: RefCell::new(None),
            sender_p2z: RefCell::new(None),
        }
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_zebra(&self, sender_p2n: mpsc::Sender<ProtoToNexus>)
                   -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ProtoToZebra>) {
        // Create channel from RouterNexus to MasterInner
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();
        let (sender_p2z, receiver_p2z) = mpsc::channel::<ProtoToZebra>();
        let handle = thread::spawn(move || {
            let mut zebra = ZebraMaster::new();
            zebra.start(sender_p2n, receiver_n2p, receiver_p2z);

            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_p2z)
    }

    // Construct MasterInner instance and spawn a thread.
    fn spawn_protocol(&self, p: ProtocolType,
                      sender_p2n: mpsc::Sender<ProtoToNexus>,
                      sender_p2z: mpsc::Sender<ProtoToZebra>)
                      -> (JoinHandle<()>, mpsc::Sender<NexusToProto>, mpsc::Sender<ZebraToProto>) {
        // Create channel from Nexus to Protocol Master
        let (sender_n2p, receiver_n2p) = mpsc::channel::<NexusToProto>();

        // Create channel from Zebra To Protocol Master
        let (sender_z2p, receiver_z2p) = mpsc::channel::<ZebraToProto>();

        let handle = thread::spawn(move || {
            let protocol = Arc::new(ProtocolMaster::new(p));
            protocol.inner_set(
                match p {
                    ProtocolType::Ospf => Box::new(OspfMasterInner::new(protocol.clone())),
                    ProtocolType::Bgp => Box::new(BgpMaster::new(protocol.clone())),
                    _ => panic!("Not supported")
                });

            protocol.timers_set(timer::Client::new(protocol.clone()));
            protocol.start(sender_p2n, receiver_n2p, sender_p2z, receiver_z2p);
            // TODO: may need some cleanup, before returning.
            ()
        });

        (handle, sender_n2p, sender_z2p)
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

    // Process command.
    fn process_command(&mut self, command: &str) -> Result<(), CoreError> {

        match command {
            "ospf" => {
                // Spawn ospf instance
                let (handle, sender, sender_z2p) =
                    self.spawn_protocol(ProtocolType::Ospf,
                                        self.clone_sender_p2n(),
                                        self.clone_sender_p2z());
                self.masters.insert(ProtocolType::Ospf, MasterTuple { handle, sender });

                // register sender_z2p to Zebra thread
            },
            "bgp" => {

            },
            "quit" => {
                return Err(CoreError::NexusTermination)
            }
            _ => {
                return Err(CoreError::CommandNotFound(command.to_string()))
            }
        }

        Ok(())
    }

    fn clone_sender_p2n(&self) -> mpsc::Sender<ProtoToNexus> {
        if let Some(ref mut sender_p2n) = *self.sender_p2n.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2n);
        }
        panic!("failed to clone");
    }

    fn clone_sender_p2z(&self) -> mpsc::Sender<ProtoToZebra> {
        if let Some(ref mut sender_p2z) = *self.sender_p2z.borrow_mut() {
            return mpsc::Sender::clone(&sender_p2z)
        }
        panic!("failed to clone");
    }

    //
    pub fn start(&mut self) {
        // Create Unix Domain Socket to accept commands.
        let mut path = env::temp_dir();
        path.push("rzrtd.cli");

        //let mut token2evented: HashMap<u32, Box<Evented>> = HashMap::new();
        let mut token2evented: HashMap<u32, RefCell<UnixStream>> = HashMap::new();

        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(_) => { panic!("UnixListener::bind() error"); }
        };

        // Create multi sender channel from MasterInner to RouterNexus
        let (sender_p2n, receiver) = mpsc::channel::<ProtoToNexus>();
        self.sender_p2n.borrow_mut().replace(sender_p2n);

        // Spawn zebra instance
        let (handle, sender, sender_p2z) = self.spawn_zebra(self.clone_sender_p2n());
        self.sender_p2z.borrow_mut().replace(sender_p2z);
        self.masters.insert(ProtocolType::Zebra, MasterTuple { handle, sender });

        // MIO poll
        let poll = Poll::new().unwrap();

        let stdin = 0;
        let stdin_fd = EventedFd(&stdin);
        poll.register(&stdin_fd, Token(0), Ready::readable(), PollOpt::level()).unwrap();
        
        poll.register(&listener, Token(1), Ready::readable(), PollOpt::level()).unwrap();

        //token2evented.insert(0, &stdin_fd);
        //token2evented.insert(1, &listener);

        'main: loop {
            // TBD: process CLI, API, poll()
            let mut events = Events::with_capacity(1024);
            poll.poll(&mut events, Some(Duration::from_millis(10))).unwrap();

            for event in events.iter() {
                match event.token() {
                    // ClI or API commands
                    Token(0) => {
                        let mut line = String::new();
                        io::stdin().lock().read_line(&mut line).unwrap();

                        let command = line.trim();

                        match self.process_command(&command) {
                            Err(CoreError::NexusTermination) => {
                                break 'main;
                            },
                            Err(CoreError::CommandNotFound(str)) => {
                                error!("Command not found '{}'", str);
                            },
                            _ => {
                            }
                        }
                    },
                    Token(1) => {
                        match listener.accept() {
                            Ok(Some((stream, _addr))) => {
                                println!("Got a client: {:?}", _addr);

                                poll.register(&stream, Token(2), Ready::readable(), PollOpt::edge()).unwrap();
                                token2evented.insert(2, RefCell::new(stream));
                            },
                            Ok(None) => println!("OK, but None???"),
                            Err(err) => println!("accept function failed: {:?}", err),
                        }
                    },
                    Token(2) => {
                        if let Some(stream) = token2evented.get(&2) {

                            let mut buffer = String::new();
                            let mut s = stream.borrow_mut();

                            s.read_to_string(&mut buffer);

                            let command = buffer.trim();

                            debug!("received command {}", command);

                            match self.process_command(&command) {
                                Err(CoreError::NexusTermination) => {
                                    break 'main;
                                },
                                Err(CoreError::CommandNotFound(str)) => {
                                    error!("Command not found '{}'", str);
                                },
                                _ => {
                                }
                            }
                        }
                    },
                    // fallback
                    _ => {
                        
                    }
                }
            }

            // Process channels
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


