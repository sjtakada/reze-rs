//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Master
//

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
//use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;

use super::cli::*;
use super::client::*;
use super::signal;
use super::config::Config;
use super::error::CliError;

use eventum::core::*;
use eventum::uds_client::*;


/// CLI Master.
pub struct CliMaster {

    /// Event Manager.
    event_manager: Arc<Mutex<EventManager>>,

    /// Remote clients.
    remote_client: Arc<Mutex<HashMap<String, Arc<dyn RemoteClient>>>>,

    /// CLI Message queue.
    message_queue: Arc<Mutex<VecDeque<CliRequest>>>,

    /// Channel to readline thread.
    sender_m2r: mpsc::Sender::<CliResponse>,
}

impl Drop for CliMaster {
    fn drop(&mut self) {
        println!("Drop CliMaster");
    }
}

unsafe impl Sync for CliMaster {}
unsafe impl Send for CliMaster {}

/// CLI Master implementation.
impl CliMaster {

    /// Constructor.
    pub fn new(sender_m2r: mpsc::Sender::<CliResponse>) -> CliMaster {
        CliMaster {
            event_manager: Arc::new(Mutex::new(EventManager::new())),
            remote_client: Arc::new(Mutex::new(HashMap::new())),
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
            sender_m2r: sender_m2r,
        }
    }

    /// Release object in CliMaster.
    pub fn release(&self) {
        self.remote_client.lock().unwrap().drain();
    }

    /// Register remote client.
    pub fn set_remote_client(&self, target: &str, client: Arc<dyn RemoteClient>) {
        self.remote_client.lock().unwrap().insert(target.to_string(), client);
    }

    /// Return remote client.
    pub fn remote_client(&self, target: &str) -> Option<Arc<dyn RemoteClient>> {
        match self.remote_client.lock().unwrap().get(target) {
            Some(client) => Some(client.clone()),
            None => None
        }
    }

    /// Return remote prefix map.
    pub fn remote_prefix_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (target, client) in self.remote_client.lock().unwrap().iter() {
            map.insert(target.to_string(), client.prefix().to_string());
        }

        map
    }

    /// Send message througm stream remote server.
    pub fn remote_send(&self, target: &str, message: &str) {
        match self.remote_client.lock().unwrap().get(target) {
            Some(client) => {
                client.stream_send(message);
            },
            None => {
                println!("No such client for {:?}", target);
            }
        }
    }

    /// Receive message through stream remote server.
    pub fn remote_recv(&self, target: &str) -> Option<String> {
        match self.remote_client.lock().unwrap().get(target) {
            Some(client) => {
                match client.stream_read() {
                    Err(err) => {
                        println!("Error: {:?}", err);
                        None
                    }
                    Ok(s) => Some(s),
                }
            },
            None => {
                None
            }
        }
    }

    /// Push a message to the queue.
    pub fn message_push(&self, message: CliRequest) {
        self.message_queue.lock().unwrap().push_back(message);
    }

    /// Start Master.
    pub fn start(config: Config) -> Result<(), CliError> {

        // Create channels.
        let (sender_r2m, receiver_r2m) = mpsc::channel::<CliRequest>();
        let (sender_m2r, receiver_m2r) = mpsc::channel::<CliResponse>();

        // Initialize master.
        let master = Arc::new(CliMaster::new(sender_m2r));
        master.init_signals()?;
        let event_manager = master.event_manager();

        // Initialize Remote clients in master context, so that they run on event manager.
        let mut config_client = Arc::new(ConfigClient::new(master.clone(), &config));
        let mut exec_client = Arc::new(ExecClient::new(master.clone(), &config));
        master.set_remote_client("config", config_client.clone());
        master.set_remote_client("exec", exec_client.clone());

        let remote_prefix_map = master.remote_prefix_map();

        // Run CLI parser in another thread.
        let handle = thread::spawn(move || {
            let mut cli = Cli::new(remote_prefix_map, sender_r2m, receiver_m2r);

            match cli.start(config) {
                Ok(_) => {},
                Err(err) => panic!("CLI Init error: {}", err),
            }

            // Notify main thread to terminate.
            cli.send_shutdown();
        });

        let cli_channel_handler = CliChannelHandler::new(master.clone(), receiver_r2m);
        event_manager.lock().unwrap().register_channel(Box::new(cli_channel_handler));

        // Event loop.
        let runner = SimpleRunner::new();
        'main: loop {
            let events = event_manager.lock().unwrap().poll();
            match runner.run(events) {
                Err(EventError::SystemShutdown) => break 'main,
                _ => {}
            }
        }

        // Release objects associated with the master.
        master.release();

        Arc::get_mut(&mut config_client).unwrap().release();
        Arc::get_mut(&mut exec_client).unwrap().release();

        event_manager.lock().unwrap().shutdown();
        drop(event_manager);

        // CLI is done.
        if let Err(err) = handle.join() {
            println!("CLI join failed {:?}", err);
        }

        Ok(())
    }

    /// Initialize signals.
    fn init_signals(&self) -> Result<(), CliError> {
        // Ignore TSTP suspend signal.
        signal::ignore_sigtstp_handler();

        Ok(())
    }

    /// Return event manager.
    pub fn event_manager(&self) -> Arc<Mutex<EventManager>> {
        self.event_manager.clone()
    }
}

/// CliRequest from readline to master.
pub enum CliRequest {
    Shutdown,
    Request((String, String)),
}

/// CliResponse from master to readline.
pub enum CliResponse {
    ServerDisconnect,
    Response((String, String)),
}

struct CliChannelHandler {

    /// CliMaster
    master: Arc<CliMaster>,

    /// Channel CliRequest receiver.
    receiver: mpsc::Receiver<CliRequest>,
}

impl CliChannelHandler {
    pub fn new(master: Arc<CliMaster>,
               receiver: mpsc::Receiver<CliRequest>)
               -> CliChannelHandler {
        CliChannelHandler {
            master: master,
            receiver: receiver,
        }
    }

    pub fn master(&self) -> Arc<dyn EventHandler> {
        self.master.clone()
    }
}

impl ChannelHandler for CliChannelHandler {
    fn poll_channel(&self) -> Vec<(EventType, Arc<dyn EventHandler>)> {
        let mut vec = Vec::new();

        while let Ok(d) = self.receiver.try_recv() {
            self.master.message_push(d);
            vec.push((EventType::ChannelEvent, self.master()));
        }

        vec
    }
}

impl EventHandler for CliMaster {

    /// Handle message.
    fn handle(&self, event_type: EventType) -> Result<(), EventError> {
        match event_type {
            EventType::ChannelEvent => {
                match self.message_queue.lock().unwrap().pop_front() {
                    Some(message) => match message {
                        CliRequest::Shutdown => Err(EventError::SystemShutdown),
                        CliRequest::Request(message) => {
                            let (target, body) = message;

                            self.remote_send(&target, &body);

                            Ok(())
                        }
                    },
                    None => Ok(())
                }
            }
            _ => {
                Err(EventError::InvalidEvent)
            }
        }
    }
}

/// UdsClientHandler for CliMaster.
impl UdsClientHandler for CliMaster {

    /// callback when client connects to server.
    fn handle_connect(&self, _entry: &UdsClient) -> Result<(), EventError> {
        println!("% Server connected.");

        Ok(())
    }

    /// callback when client detects server disconnected.
    fn handle_disconnect(&self, _entry: &UdsClient) -> Result<(), EventError> {
        println!("% Server disconncted.");

        Ok(())
    }

    /// callback when client received message.
    fn handle_message(&self, entry: &UdsClient) -> Result<(), EventError> {
        let resp = entry.stream_read()?;
        let target = String::from("TBD");

        // Send response back to readline.
        self.sender_m2r.send(CliResponse::Response((target, resp)));

        Ok(())
    }
}
