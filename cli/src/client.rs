//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Config and Exec Client.
//

use std::env;
use std::sync::Arc;
use std::sync::Mutex;

use eventum::core::*;
use eventum::uds_client::*;

use common::consts::*;

use super::master::CliMaster;
use super::config::Config;

/// Trait Remote Client.
pub trait RemoteClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<Mutex<UdsClient>>;

    /// Return API prefix.
    fn prefix(&self) -> &str;

    /// Connect config server.
    fn connect(&self) {
        self.uds_client().lock().unwrap().connect();
    }

    /// Send message to config server.
    fn stream_send(&self, message: &str) {
        if let Err(err) = self.uds_client().lock().unwrap().stream_send(message) {
            println!("% Stream send error {:?}", err);
        }
    }

    /// Recv message from config server.
    fn stream_read(&self) -> Result<String, EventError> {
        self.uds_client().lock().unwrap().stream_read()
    }
}

/// Config client.
pub struct ConfigClient {

    /// UDS client.
    uds_client: Option<Arc<Mutex<UdsClient>>>,

    /// API path prefix.
    prefix: String,
}

impl Drop for ConfigClient {
    fn drop(&mut self) {
        println!("Drop ConfigClient");
    }
}

/// Config client implementation.
impl ConfigClient {

    /// Constructor.
    pub fn new(master: Arc<CliMaster>, config: &Config) -> ConfigClient {

        let mut path = env::temp_dir();
        let socket_file = if let Some(remote) = config.remote("config") {
            remote.uds_socket_file()
        } else {
            None
        };

        match socket_file {
            Some(socket_file) => path.push(socket_file),
            None => path.push(ROUTERD_CONFIG_UDS_FILENAME),
        }

        let uds_client = UdsClient::start(master.event_manager(), master.clone(), &path);
        uds_client.lock().unwrap().connect();

        let prefix = match config.remote("config") {
            Some(remote) => {
                match remote.prefix() {
                    Some(prefix) => prefix.to_string(),
                    None => ROUTERD_CONFIG_API_PREFIX.to_string(),
                }
            },
            None => ROUTERD_CONFIG_API_PREFIX.to_string(),
        };

        ConfigClient {
            uds_client: Some(uds_client),
            prefix: prefix
        }
    }

    pub fn release(&mut self) {
        self.uds_client = None;
    }
}

/// RemoteClient implementation for ConfigClient.
impl RemoteClient for ConfigClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<Mutex<UdsClient>> {
        match &self.uds_client {
            Some(client) => client.clone(),
            None => panic!("Uds Client doesn't exist"),
        }
    }

    /// Return API prefix.
    fn prefix(&self) -> &str {
        &self.prefix
    }
}


/// Exec client.
pub struct ExecClient {

    /// UDS client.
    uds_client: Option<Arc<Mutex<UdsClient>>>,

    /// API path prefix.
    prefix: String,
}

impl Drop for ExecClient {
    fn drop(&mut self) {
        println!("Drop ExecClient");
    }
}

/// Exec client implementation.
impl ExecClient {

    /// Constructor.
    pub fn new(master: Arc<CliMaster>, config: &Config) -> ExecClient {

        let mut path = env::temp_dir();
        let socket_file = if let Some(remote) = config.remote("exec") {
            remote.uds_socket_file()
        } else {
            None
        };

        match socket_file {
            Some(socket_file) => path.push(socket_file),
            None => path.push(ROUTERD_EXEC_UDS_FILENAME),
        }

        let uds_client = UdsClient::start(master.event_manager(), master.clone(), &path);
        uds_client.lock().unwrap().connect();

        let prefix = match config.remote("exec") {
            Some(remote) => {
                match remote.prefix() {
                    Some(prefix) => prefix.to_string(),
                    None => ROUTERD_EXEC_API_PREFIX.to_string(),
                }
            },
            None => ROUTERD_EXEC_API_PREFIX.to_string(),
        };

        ExecClient {
            uds_client: Some(uds_client),
            prefix: prefix
        }
    }

    pub fn release(&mut self) {
        self.uds_client = None;
    }
}

/// RemoteClient implementation for ExecClient.
impl RemoteClient for ExecClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<Mutex<UdsClient>> {
        match &self.uds_client {
            Some(client) => client.clone(),
            None => panic!("Uds Client doesn't exist"),
        }
    }

    /// Return API prefix.
    fn prefix(&self) -> &str {
        &self.prefix
    }
}
