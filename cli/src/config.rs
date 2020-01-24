//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Global CLI configuration.
//

use std::default::Default;
use std::collections::HashMap;

/// Global CLI configuration, populated through command line or file.
pub struct Config {

    /// CLI debug mode.
    debug: bool,

    /// CLI defintion JSON directory.
    cli_definition_dir: Option<String>,

    /// Configs for remote endpoint.
    remote: HashMap<String, ConfigRemote>,
}
    
/// Global CLI config.
impl Config {

    /// Constructor.
    pub fn new() -> Config {
        Config::default()
    }

    /// Parse config from JSON.
    pub fn from_json(json: &serde_json::Value) -> Config {
        let mut config = Config::new();

        if json.is_object() {
            for k in json.as_object().unwrap().keys() {
                match k.as_ref() {
                    "debug" => {
                        if let Some(v) = json.get(k).unwrap().as_bool() {
                            config.set_debug(v);
                        }
                    },
                    "cli_definition" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_cli_definition_dir(v);
                        }
                    },
                    "remote" => {
                        if let Some(v) = json.get(k).unwrap().as_object() {
                            for name in v.keys() {
                                let remote = match config.remote(name) {
                                    Some(r) => r,
                                    None => {
                                        config.set_remote(name);
                                        config.remote(name).unwrap()
                                    }
                                };

                                remote.from_json(v.get(name).unwrap());
                            }
                        }
                    },
                    "description" => {},
                    "config_schema_version" => {},
                    _ => {
                        println!("Unknown keyword in global config {:?}", k);
                    }
                }
            }
        }

        config
    }

    /// Return debug flag.
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// Return CLI definition directory.
    pub fn cli_definition_dir(&self) -> Option<&str> {
        self.cli_definition_dir.as_ref().map(|s| &s[..])
    }

    /// Return config for remote endpoint.
    pub fn remote(&self, name: &str) -> Option<&ConfigRemote> {
        self.remote.get(name)
    }

    /// Set debug flag.
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Set CLI definition directory.
    pub fn set_cli_definition_dir(&mut self, cli_definition_dir: &str) {
        self.cli_definition_dir.replace(String::from(cli_definition_dir));
    }

    /// Set config for remote endpoint.
    pub fn set_remote(&mut self, name: &str) {
        self.remote.insert(String::from(name), ConfigRemote::default());
    }
}

/// Default implementation for Config.
impl Default for Config {

    /// Return instance with default value.
    fn default() -> Self {
        Self {
            debug: false,
            cli_definition_dir: Some(String::from("./json")),
            remote: HashMap::new(),
        }
    }
}

/// Remote endpoint config.
pub struct ConfigRemote {

    /// Transport.
    transport: Option<String>,

    /// Socket path for UNIX domain socket.
    socket: Option<String>,

    /// Server IP for TCP.
    server_ip: Option<String>,

    /// Server port.
    server_port: Option<u16>,

    /// Protocol, data format of remote access.
    protocol: Option<String>,

    /// Prefix for API.
    prefix: Option<String>,

 // Authentication.
}

/// ConfigRemote implementation.
impl ConfigRemote {

    /// Constructor.
    pub fn new() -> ConfigRemote {
        ConfigRemote::default()
    }

    /// Parse config from JSON.
    pub fn from_json(&self, json: &serde_json::Value) -> ConfigRemote {
        let mut config = ConfigRemote::new();

        if json.is_object() {
            for k in json.as_object().unwrap().keys() {
                match k.as_ref() {
                    "transport" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_transport(v);
                        }
                    },
                    "socket" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_socket(v);
                        }
                    },
                    "server_ip" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_server_ip(v);
                        }
                    },
                    "server_port" => {
                        if let Some(v) = json.get(k).unwrap().as_u64() {
                            config.set_server_port(v as u16);
                        }
                    },
                    "protocol" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_protocol(v);
                        }
                    },
                    "prefix" => {
                        if let Some(v) = json.get(k).unwrap().as_str() {
                            config.set_prefix(v);
                        }
                    },
                    "authentication" => {},
                    _ => {
                        println!("Unknown keyword in remote config {:?}", k);
                    }
                }
            }
        }

        config
    }

    /// Return transport.
    pub fn transport(&self) -> Option<&str> {
        self.transport.as_ref().map(|s| &s[..])
    }

    /// Return socket path for UNIX domain socket.
    pub fn socket(&self) -> Option<&str> {
        self.socket.as_ref().map(|s| &s[..])
    }

    /// Return server IP for TCP.
    pub fn server_ip(&self) -> Option<&str> {
        self.server_ip.as_ref().map(|s| &s[..])
    }

    /// Return server port.
    pub fn server_port(&self) -> Option<u16> {
        self.server_port
    }

    /// Return protocol, data format of remote access.
    pub fn protocol(&self) -> Option<&str> {
        self.protocol.as_ref().map(|s| &s[..])
    }

    /// Return prefix for API.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_ref().map(|s| &s[..])
    }

    /// Set transport.
    pub fn set_transport(&mut self, transport: &str) {
        self.transport.replace(String::from(transport));
    }

    /// Set socket path for UNIX domain socket.
    pub fn set_socket(&mut self, socket: &str) {
        self.socket.replace(String::from(socket));
    }

    /// Set server IP for TCP.
    pub fn set_server_ip(&mut self, server_ip: &str) {
        self.server_ip.replace(String::from(server_ip));
    }

    /// Set server port.
    pub fn set_server_port(&mut self, server_port: u16) {
        self.server_port.replace(server_port);
    }

    /// Set protocol, data format of remote access.
    pub fn set_protocol(&mut self, protocol: &str) {
        self.protocol.replace(String::from(protocol));
    }

    /// Set prefix for API.
    pub fn set_prefix(&mut self, prefix: &str) {
        self.prefix.replace(String::from(prefix));
    }

    /// Return UDS socket filename.
    pub fn uds_socket_file(&self) -> Option<&str> {
        if let Some(transport) = &self.transport {
            if transport == "unix" {
                if let Some(socket) = &self.socket {
                    return Some(&socket[..])
                }
            }
        }

        None
    }
}

/// Default implementation for ConfigRemote.
impl Default for ConfigRemote {

    /// Return instance with default value.
    fn default() -> Self {
        Self {
            transport: None,
            socket: None,
            server_ip: None,
            server_port: None,
            protocol: None,
            prefix: None,
        }
    }
}
