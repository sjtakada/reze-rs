//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// CLI - Core Shell functions.
//

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use serde_json;
use rustyline::error::ReadlineError;

use super::client::*;
use super::utils::*;
use super::config::Config;
use super::error::CliError;
use super::readline::*;
use super::tree::CliTree;
use super::builtins;

// Constants.
const CLI_INITIAL_MODE: &str = "EXEC-MODE";
const CLI_MODE_FILE: &str = "reze.cli_mode.json";

//
// Main container of CLI
//
pub struct Cli {

    /// HashMap from mode name to CLI tree.
    trees: HashMap<String, Rc<CliTree>>,

    /// Built-in functions.
    builtins: HashMap<String, Box<dyn Fn(&Cli, &Vec<String>) -> Result<(), CliError>>>,

    /// Current mode name.
    mode: RefCell<String>,

    /// Prompt.
    prompt: RefCell<String>,

    /// Current privilege.
    privilege: Cell<u8>,

    /// Remote clients.
    remote_client: RefCell<HashMap<String, Arc<dyn RemoteClient>>>,

    /// Debug mode.
    debug: bool,
}

/// CLI.
impl Cli {

    /// Constructor.
    pub fn new() -> Cli {
        Cli {
            trees: HashMap::new(),
            builtins: HashMap::new(),
            mode: RefCell::new(String::new()),
            prompt: RefCell::new(String::new()),
            privilege: Cell::new(1),
            remote_client: RefCell::new(HashMap::new()),
            debug: false,
        }
    }

    /// Register remote client.
    pub fn set_remote_client(&self, target: &str, client: Arc<dyn RemoteClient>) {
        self.remote_client.borrow_mut().insert(target.to_string(), client);
    }

    /// Return remote client.
    pub fn remote_client(&self, target: &str) -> Option<Arc<dyn RemoteClient>> {
        match self.remote_client.borrow_mut().get(target) {
            Some(client) => Some(client.clone()),
            None => None
        }
    }

    /// Entry point of shell initialization.
    pub fn start(&mut self, config: Config) -> Result<(), CliError> {

        self.debug = config.debug();
        if self.debug {
            println!("% Debug mode");
        }

        // TBD: Terminal init

        // Initialize CLI modes.
        let mut path = PathBuf::from(config.cli_definition_dir().unwrap());
        path.push(CLI_MODE_FILE);
        self.init_cli_modes(&path)?;

        // Initialize build-in commands.
        self.init_builtins()?;

        // Initialize CLI comand definitions.
        let path = PathBuf::from(config.cli_definition_dir().unwrap());
        self.init_cli_commands(&path)?;
        self.set_mode(CLI_INITIAL_MODE)?;

        // Init readline.
        let readline = CliReadline::new(&self);

        // Start CLI.
        self.run(readline);

        // Successfully finished.
        Ok(())
    }

    /// Run command loop.
    pub fn run(&self, readline: CliReadline) {
        loop {
            // TODO, we'll get API URL and parameters here to send to server.
            match readline.gets() {
                Ok(line) => {
                    readline.execute(line);
                },
                Err(ReadlineError::Interrupted) => {
                    // do nothing
                },
                Err(ReadlineError::Eof) => {
                    if self.can_exit() {
                        break
                    }

                    readline.execute(String::from("end"));
                },
                Err(ReadlineError::Suspended) => {
                    self.config_end();
                },
                Err(err) => {
                    println!("Error: {:?}", err);
                }
            };
        }
    }

    /// TBD: probably should be initialized in builtins.rs.
    fn init_builtins(&mut self) -> Result<(), CliError> {
        self.builtins.insert("help".to_string(), Box::new(builtins::help));
        self.builtins.insert("exit".to_string(), Box::new(builtins::exit));
        self.builtins.insert("enable".to_string(), Box::new(builtins::enable));
        self.builtins.insert("disable".to_string(), Box::new(builtins::disable));
        self.builtins.insert("show_privilege".to_string(), Box::new(builtins::show_privilege));

        Ok(())
    }

    pub fn call_builtin(&self, func: &str, params: &Vec<String>) -> Result<(), CliError> {
        match self.builtins.get(func) {
            Some(func) => {
                func(self, params).unwrap();
                Ok(())
            },
            None => {
                Err(CliError::ActionError(format!("builtin '{}'", func)))
            }
        }
    }

    fn can_exit(&self) -> bool {
        let mode = self.mode.borrow_mut();
        if String::from(mode.as_str()) == CLI_INITIAL_MODE {
            true
        }
        else {
            false
        }
    }

    fn config_end(&self) {
        self.set_mode(CLI_INITIAL_MODE).expect("Failed to set mode");
    }

    pub fn trees(&self) -> &HashMap<String, Rc<CliTree>> {
        &self.trees
    }

    pub fn mode(&self) -> String {
        let mode = self.mode.borrow_mut();
        String::from(mode.as_str())
    }

    pub fn set_privilege(&self, privilege: u8) {
        self.privilege.set(privilege);
    }

    pub fn privilege(&self) -> u8 {
        self.privilege.get()
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }

    pub fn current(&self) -> Option<Rc<CliTree>> {
        match self.trees.get(&self.mode()) {
            Some(tree) => Some(tree.clone()),
            None => None,
        }
    }

    // TODO: hostname, consider return reference.
    pub fn set_prompt(&self) {
        let mut prompt = String::from("Router");
        let current = self.current().unwrap();
        if current.prompt().len() > 0 {
            prompt.push_str(current.prompt());
        }
        if self.privilege.get() > 1 {
            prompt.push_str("#");
        }
        else {
            prompt.push_str(">");
        }

        self.prompt.replace(prompt);
    }

    pub fn prompt(&self) -> String {
        self.prompt.borrow_mut().clone()
    }

    pub fn has_parent(&self) -> bool {
        match self.current().unwrap().parent() {
            Some(_parent) => true,
            None => false
        }
    }

    pub fn set_mode(&self, mode: &str) -> Result<(), CliError> {
        self.mode.replace(String::from(mode));
        self.set_prompt();

        Ok(())
    }

    pub fn set_mode_up(&self) -> Result<(), CliError> {
        let current = self.current().unwrap();
        if let Some(parent) = current.parent() {
            self.set_mode(parent.name()).unwrap();
        }

        Ok(())
    }

    // Initialize CLI modes.
    fn init_cli_modes(&mut self, path: &Path) -> Result<(), CliError> {
        match json_read(path) {
            Some(root) => {
                if root.is_object() {
                    self.build_mode(&root, None)?;
                }
            },
            None => return Err(CliError::InitModeError),
        }

        Ok(())
    }

    // Build CLI mode tree from JSON.
    fn build_mode(&mut self, json: &serde_json::Value, parent: Option<Rc<CliTree>>) -> Result<(), CliError> {
        for name in json.as_object().unwrap().keys() {
            let mode = &json[name];
            if mode.is_object() {
                let prompt = if mode["prompt"].is_string() {
                    &mode["prompt"].as_str().unwrap()
                } else {
                    ""
                };
                let children = &mode["children"];
                let tree = Rc::new(CliTree::new(name.to_string(), prompt.to_string(), parent.clone()));
                self.trees.insert(name.to_string(), tree.clone());

                if children.is_object() {
                    self.build_mode(&children, Some(tree.clone()))?;
                }
            }
        }

        Ok(())
    }

    fn parse_defun(&mut self, defun_tokens: &serde_json::Value,
                   command: &serde_json::Value) {
        if command["mode"].is_array() {
            for mode in command["mode"].as_array().unwrap() {
                if let Some(mode) = mode.as_str() {
                    if let Some(tree) = self.trees.get(mode) {
                        tree.build_command(defun_tokens, command);
                    }
                }
            }
        }
    }

    fn parse_defun_all(&mut self, defun_tokens: &serde_json::Value,
                       commands: &serde_json::Value) {
        if defun_tokens.is_object() && commands.is_array() {
            let commands = commands.as_array().unwrap();
            for command in commands {
                self.parse_defun(&defun_tokens, &command);
            }
        }
    }

    fn load_cli_json(&mut self, path: &Path) {
        if let Some(json) = json_read(path) {
            if json.is_object() {
                for k in json.as_object().unwrap().keys() {
                    if let Some(attr) = json[k].as_object() {
                        self.parse_defun_all(&attr["token"], &attr["command"]);
                    }
                }
            }
        }
    }

    fn init_cli_commands(&mut self, dir: &Path) -> Result<(), CliError> {
        // Right now only read
        //   filename does not start with '_' and
        //   filename ends with '.cli.json'.
        if dir.is_dir() {
            for entry in fs::read_dir(dir).expect("Unable to get directory entry") {
                let entry = entry.expect("Unable to get an entry");
                let path = entry.path();

                if let Some(filename) = path.file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if !filename_str.starts_with("_") && filename_str.ends_with(".cli.json") {
                            self.load_cli_json(&path);
                        }
                    }
                }
            }

            for tree in self.trees.values() {
                tree.sort();
            }
        }

        Ok(())
    }

    /// Send message througm stream remote server.
    pub fn remote_send(&self, target: &str, message: &str) {
        match self.remote_client.borrow_mut().get(target) {
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
        match self.remote_client.borrow_mut().get(target) {
            Some(client) => {
                client.stream_read()
            },
            None => {
                None
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const TMP_FILE: &str = "/tmp/clitest.json";

    fn write_tmp_file(data: &[u8]) {
        fs::write(TMP_FILE, data).expect("Unable to write file");
    }

    fn rm_tmp_file() {
        let _ = fs::remove_file(TMP_FILE);
    }

    #[test]
    pub fn test_json_read() {
        let pathbuf = PathBuf::from(TMP_FILE);

        // No file written yet.
        let ret = json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // Write non UTF text to file.
        let no_utf_txt = &[0xe9, 0x5a, 0xe9, 0x4a];
        write_tmp_file(no_utf_txt);
        let ret = json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // UTF but not JSON.
        let utf_txt = "饂飩";
        write_tmp_file(utf_txt.as_bytes());
        let ret = json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // Proper UTF JSON.
        let json_txt = "{\"noodle\":\"饂飩\"}";
        write_tmp_file(json_txt.as_bytes());
        let ret = json_read(pathbuf.as_path());
        let v = serde_json::from_str(json_txt).unwrap();
        assert_eq!(ret, v);

        rm_tmp_file();
    }

    //fn mode_lists

    #[test]
    pub fn test_cli_modes() {
        let mut cli = Cli::new();
        let mode_json_str = r##"
{
  "ENABLE-MODE": {
    "prompt": "#"
  },
  "CONFIG-MODE": {
    "prompt": "(config)#",
    "children": {
      "EMPTY-MODE": {
      },
      "EMPTY-CHILDREN": {
        "children": {
        } 
      },
      "INTERFACE-MODE": {
        "prompt": "(config-if)#"
      },
      "BGP-MODE": {
        "prompt": "(config-router)#",
        "children": {
          "BGP-AF-MODE": {
            "prompt": "(config-router-af)#"
          }
        }
      }
    }
  }
} "##;

        let path = PathBuf::from("../json/reze.cli_mode.json");
        let _ret = cli.init_cli_modes(&path);
        let json = serde_json::from_str(&mode_json_str).unwrap();
        let _ret = cli.build_mode(&json, None);
        let mode = &cli.trees["BGP-AF-MODE"];
        let mode = mode.parent().unwrap();
        assert_eq!(mode.name(), "BGP-MODE");
        let mode = mode.parent().unwrap();
        assert_eq!(mode.name(), "CONFIG-MODE");
        assert_eq!(mode.prompt(), "(config)#");
        let mode = mode.parent();
        assert_eq!(match mode { None => true, _ => false } , true);
    }
}
