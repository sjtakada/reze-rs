//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI Master
//

use std::env;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::cell::RefCell;
use std::time::Duration;

use common::error::*;
use common::consts::*;
use common::event::*;
use common::uds_client::*;

use super::cli::*;
use super::signal;
use super::config::Config;
use super::error::CliError;

/// CLI Master.
pub struct CliMaster {

    /// Event Manager.
    event_manager: RefCell<Arc<EventManager>>,

    /// UDS Client.
    uds_client: RefCell<Option<Arc<UdsClient>>>,
}

/// CLI Master implementation.
impl CliMaster {

    /// Constructor.
    pub fn new() -> CliMaster {
        CliMaster {
            event_manager: RefCell::new(Arc::new(EventManager::new())),
            uds_client: RefCell::new(None),
        }
    }

    /// Start Master.
    pub fn start(config: Config) -> Result<(), CliError> {
        // Initialize master and UDS client.
        let master = Arc::new(CliMaster::new());
        master.init_signals()?;

        let mut path = env::temp_dir();
        path.push(ROUTERD_CONFIG_UDS_FILENAME);
        let event_manager = master.event_manager.borrow_mut();
        let client = UdsClient::start(event_manager.clone(), master.clone(), &path);
        master.uds_client.borrow_mut().replace(client.clone());

        let (sender, receiver) = mpsc::channel::<bool>();

        // Run CLI parser in another thread.
        let handle = thread::spawn(move || {
            let mut cli = Cli::new(client.clone());
            match cli.start(config) {
                Ok(_) => {},
                Err(err) => panic!("CLI Init error: {}", err),
            }

            sender.send(true).unwrap();
        });

        // Event loop.
        'main: loop {
            match event_manager.poll() {
                Err(_err) => break 'main,
                _ => {}
            }

            thread::sleep(Duration::from_millis(EVENT_MANAGER_TICK));

            if let Ok(_) = receiver.try_recv() {
                break 'main
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

