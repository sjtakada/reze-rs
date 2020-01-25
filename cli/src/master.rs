//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Master
//

use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::cell::RefCell;

use common::error::*;
use common::event::*;
use common::uds_client::*;

use super::cli::*;
use super::client::*;
use super::signal;
use super::config::Config;
use super::error::CliError;

/// CLI Master.
pub struct CliMaster {

    /// Event Manager.
    event_manager: RefCell<Arc<EventManager>>,
}

/// CLI Master implementation.
impl CliMaster {

    /// Constructor.
    pub fn new() -> CliMaster {
        CliMaster {
            event_manager: RefCell::new(Arc::new(EventManager::new())),
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

        let (sender, receiver) = mpsc::channel::<bool>();

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
            sender.send(true).unwrap();
        });

        let handler = move |_event_manager: &EventManager| -> Result<(), CoreError> {
            if let Ok(_) = receiver.try_recv() {
                Err(CoreError::SystemShutdown)
            } else {
                Ok(())
            }
        };
        event_manager.set_channel_handler(Box::new(handler));

        // Event loop.
        'main: loop {
            match event_manager.run() {
                Err(CoreError::SystemShutdown) => break 'main,
                _ => {},
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
    pub fn event_manager(&self) -> Arc<EventManager> {
        self.event_manager.borrow_mut().clone()
    }
}

/// UdsClientHandler for CliMaster.
impl UdsClientHandler for CliMaster {

    /// callback when client connects to server.
    fn handle_connect(&self, /*client: Arc<UdsClient>, */_entry: &UdsClient) -> Result<(), CoreError> {
        println!("Server conncted.");

        Ok(())
    }

    /// callback when client detects server disconnected.
    fn handle_disconnect(&self, /*client: Arc<UdsClient>, */_entry: &UdsClient) -> Result<(), CoreError> {
        println!("Server disconncted.");
        // Should restart reconnect timer.

        Ok(())
    }

    /// callback when client received message.
    fn handle_message(&self, /*client: Arc<UdsClient>, */_entry: &UdsClient) -> Result<(), CoreError> {
        Ok(())
    }
}

/// UDS client connect timer.
pub struct UdsClientTimer {

}
