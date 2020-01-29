//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Config and Exec Client.
//

use std::env;
use std::sync::Arc;

use common::consts::*;
use common::uds_client::UdsClient;

use super::master::CliMaster;
use super::config::Config;

/// Trait Remote Client.
pub trait RemoteClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<UdsClient>;

    /// Return API prefix.
    fn prefix(&self) -> &str;

    /// Connect config server.
    fn connect(&self) {
        self.uds_client().connect();
    }

    /// Send message to config server.
    fn stream_send(&self, message: &str) {
        if let Err(err) = self.uds_client().stream_send(message) {
            println!("% Stream send error {:?}", err);
        }
    }

    /// Recv message from config server.
    fn stream_read(&self) -> Option<String> {
        self.uds_client().stream_read()
    }
}

/// Config client.
pub struct ConfigClient {

    /// UDS client.
    uds_client: Arc<UdsClient>,

    /// API path prefix.
    prefix: String,
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
        uds_client.connect();

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
            uds_client: uds_client,
            prefix: prefix
        }
    }
}

/// RemoteClient implementation for ConfigClient.
impl RemoteClient for ConfigClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<UdsClient> {
        self.uds_client.clone()
    }

    /// Return API prefix.
    fn prefix(&self) -> &str {
        &self.prefix
    }
}


/// Exec client.
pub struct ExecClient {

    /// UDS client.
    uds_client: Arc<UdsClient>,

    /// API path prefix.
    prefix: String,
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
        uds_client.connect();

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
            uds_client: uds_client,
            prefix: prefix
        }
    }
}

/// RemoteClient implementation for ExecClient.
impl RemoteClient for ExecClient {

    /// Return UDS client.
    fn uds_client(&self) -> Arc<UdsClient> {
        self.uds_client.clone()
    }

    /// Return API prefix.
    fn prefix(&self) -> &str {
        &self.prefix
    }
}
