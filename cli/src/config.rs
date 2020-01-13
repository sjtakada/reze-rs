//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Global CLI configuration.
//

use std::default::Default;

// Global CLI configuration, populated through command line or file.
pub struct Config {
    // CLI debug mode.
    debug: bool,

    // CLI defintion JSON directory.
    json: Option<String>,

    // API server IP address.
    server_ip: Option<String>,

    // API prefix for CliActionHttp.
    api_prefix: Option<String>,

    // Username and password for API.
    user_pass: Option<String>,
}
    
impl Config {
    pub fn new() -> Config {
        Config::default()
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn json(&self) -> Option<&str> {
        self.json.as_ref().map(|s| &s[..])
    }

    pub fn server_ip(&self) -> Option<&str> {
        self.server_ip.as_ref().map(|s| &s[..])
    }

    pub fn api_prefix(&self) -> Option<&str> {
        self.api_prefix.as_ref().map(|s| &s[..])
    }

    pub fn user_pass(&self) -> Option<&str> {
        self.user_pass.as_ref().map(|s| &s[..])
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn set_json(&mut self, json: &str) {
        self.json.replace(String::from(json));
    }

    pub fn set_server_ip(&mut self, server_ip: &str) {
        self.server_ip.replace(String::from(server_ip));
    }

    pub fn set_api_prefix(&mut self, api_prefix: &str) {
        self.api_prefix.replace(String::from(api_prefix));
    }

    pub fn set_user_pass(&mut self, user_pass: &str) {
        self.user_pass.replace(String::from(user_pass));
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: true,
            json: Some(String::from(".")),
            server_ip: Some(String::from("localhost")),
            api_prefix: Some(String::from("/")),
            user_pass: None,
        }
    }
}

