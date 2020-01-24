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

        let prefix = String::from("");

        ConfigClient {
            uds_client: uds_client,
            prefix: prefix
        }
    }

    /// Connect config server.
    pub fn connect(&self) {
        self.uds_client.connect();
    }

    /// Send message to config server.
    pub fn stream_send(&self, message: &str) {
        self.uds_client.stream_send(message);
    }
}
