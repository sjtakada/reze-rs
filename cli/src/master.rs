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

use super::cli::*;
use super::client::*;
use super::signal;
use super::config::Config;
use super::error::CliError;
use super::uds_client::*;

use eventum::*;


/// CLI Master.
pub struct CliMaster {

    /// Event Manager.
    event_manager: Arc<Mutex<EventManager>>,
}

/// CLI Master implementation.
impl CliMaster {

    /// Constructor.
    pub fn new() -> CliMaster {
        CliMaster {
            event_manager: Arc::new(Mutex::new(EventManager::new())),
        }
    }

    /// Start Master.
    pub fn start(config: Config) -> Result<(), CliError> {

        // Initialize master.
        let master = Arc::new(CliMaster::new());
        master.init_signals()?;
        let event_manager = master.event_manager();

        // Initialize Remote clients in master context, so that they run on event manager.
        let config_client = Arc::new(ConfigClient::new(master.clone(), &config));
        let exec_client = Arc::new(ExecClient::new(master.clone(), &config));

        let (sender, receiver) = mpsc::channel::<CliMessage>();

        // Run CLI parser in another thread.
        let handle = thread::spawn(move || {
            let mut cli = Cli::new();
            cli.set_remote_client("config", config_client.clone());
            cli.set_remote_client("exec", exec_client.clone());

            match cli.start(config) {
                Ok(_) => {},
                Err(err) => panic!("CLI Init error: {}", err),
            }

            // Notify main thread to terminate.
            sender.send(CliMessage { shutdown: true }).unwrap();
        });

        let cli_channel_handler = CliChannelHandler { receiver };
        event_manager.lock().unwrap().register_channel(Box::new(cli_channel_handler));

        let runner = SimpleRunner::new();

        // Event loop.
        'main: loop {
            let events = event_manager.lock().unwrap().poll();
            match runner.run(events) {
                Err(EventError::SystemShutdown) => break 'main,
                _ => {}
            }
        }

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

/// CliMessage for shutdown.
struct CliMessage {
    shutdown: bool,
}

struct CliChannelHandler {
    receiver: mpsc::Receiver<CliMessage>,
}

impl CliChannelHandler {
    pub fn new(receiver: mpsc::Receiver<CliMessage>) -> CliChannelHandler {
        CliChannelHandler {
            receiver: receiver,
        }
    }
}

impl ChannelHandler for CliChannelHandler {
    fn poll_channel(&self) -> Vec<(EventType, Arc<dyn EventHandler>)> {
        let mut vec = Vec::new();

        while let Ok(d) = self.receiver.try_recv() {
            let handler = ShutdownMessageHandler::new(d);
            vec.push((EventType::ChannelEvent, handler));
        }

        vec
    }
}

struct ShutdownMessageHandler {
    _message: CliMessage,
}

impl ShutdownMessageHandler {
    pub fn new(message: CliMessage) -> Arc<dyn EventHandler> {
        Arc::new(ShutdownMessageHandler {
            _message: message,
        })
    }
}

impl EventHandler for ShutdownMessageHandler {

    /// Handle message.
    fn handle(&self, event_type: EventType) -> Result<(), EventError> {
        match event_type {
            EventType::ChannelEvent => {
                Err(EventError::SystemShutdown)
            }
            _ => {
                Ok(())
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
    fn handle_disconnect(&self, entry: &UdsClient) -> Result<(), EventError> {
        println!("% Server disconncted.");
        // Should restart reconnect timer.

//        entry.connect_timer();

        Ok(())
    }

    /// callback when client received message.
    fn handle_message(&self, entry: &UdsClient) -> Result<(), EventError> {
//        let inner = entry.get_inner();

        entry.stream_read()?;

//        if let Err(_err) = entry.stream_read() {
//            self.handle_disconnect(entry)?;
//        }

        Ok(())
    }
}
