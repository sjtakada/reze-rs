//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Main
//

use std::env;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use mio_uds::UnixStream;
use serde_json;
//use serde_json::map::*;

use super::error::CliError;
use super::readline::*;
use super::tree::CliTree;

pub struct Cli {
    // HashMap from mode name to CLI tree.
    trees: HashMap<String, Rc<CliTree>>,

    // Readline.
    readline: RefCell<CliReadline>,
}

impl Cli {
    pub fn new() -> Cli {
        Cli {
            trees: HashMap::new(),
            readline: RefCell::new(CliReadline::new()),
        }
    }

    // Entry point of shell initialization.
    pub fn init(&mut self) -> Result<(), CliError> {
        // TBD: Signal init
        // TBD: Terminal init

        // Initialize CLI modes.
        self.init_cli_modes()?;

        // Initialize build-in commands.

        // Initialize CLI definitions.
        let mut path = PathBuf::from("../json");
        self.init_cli_commands(&path);

        // TBD: Connect server or send.
        self.init_server_connect()?;

        // Init readline.

        Ok(())
    }

    pub fn run(&self) {
        loop {
            // TODO, we'll get API URL and parameters here to send to server.
            self.readline.borrow_mut().gets();

            /*
            stdout().write(b"> ");
            stdout().flush();

            let mut buffer = String::new();
            stdin().read_line(&mut buffer);

            stream.write(buffer.as_ref());
            stream.flush();
             */
        }
    }

    // Read and return JSON, if it fails, return None.
    fn json_read(&self, path: &Path) -> Option<serde_json::Value> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                println!("Unable to open file: {:?}: {:?}", path, err);
                return None
            }
        };

        let mut buf_reader = BufReader::new(file);
        let mut json_str = String::new();
        match buf_reader.read_to_string(&mut json_str) {
            Ok(_) => {},
            Err(err) => {
                println!("Unable to read file: {:?}: {:?}", path, err);
                return None
            }
        };

        match serde_json::from_str(&json_str) {
            Ok(value) => value,
            Err(err) => {
                println!("Unable to parse string as JSON: {:?}: {:?}", path, err);
                None
            }
        }
    }

    // Initialize CLI modes.
    fn init_cli_modes(&mut self) -> Result<(), CliError> {
        let pathbuf = PathBuf::from("../json/reze.cli_mode.json");
        match self.json_read(pathbuf.as_path()) {
            Some(root) => {
                if root.is_object() {
                    self.build_mode(&root, None);
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
                    ">"
                };
                let children = &mode["children"];
                let tree = Rc::new(CliTree::new(name.to_string(), prompt.to_string(), parent.clone()));
                self.trees.insert(name.to_string(), tree.clone());

                if children.is_object() {
                    self.build_mode(&children, Some(tree.clone()));
                }
            }
        }

        Ok(())
    }

    fn parse_defun(&mut self, tokens: &serde_json::Value,
                   command: &serde_json::Value) {
        if command["mode"].is_array() {
            for mode in command["mode"].as_array().unwrap() {
                if let Some(mode) = mode.as_str() {
                    if let Some(tree) = self.trees.get(mode) {
                        tree.build_command(tokens, command);
                    }
                }
            }
        }
    }

    fn parse_defun_all(&mut self, tokens: &serde_json::Value,
                       commands: &serde_json::Value) {
        if tokens.is_object() && commands.is_array() {
            let commands = commands.as_array().unwrap();
            for command in commands {
                self.parse_defun(&tokens, &command);
            }
        }
    }

    fn load_cli_json(&mut self, path: &Path) {
        if let Some(json) = self.json_read(path) {
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
        let suffix = "cli.json";

        if dir.is_dir() {
            for entry in fs::read_dir(dir).expect("Unable to get directory entry") {
                let entry = entry.expect("Unable to get an entry");
                let path = entry.path();

                if path.is_file() {
                    if let Some(path_str) = path.to_str() {
                        if path_str.ends_with(".cli.json") {
                            self.load_cli_json(&path);
                        }
                    }
                }

                // TBD
                break;
            }
        }

        // TBD
        panic!("hoge");

        Ok(())
    }

    fn init_server_connect(&self) -> Result<(), CliError> {
        // Initialize connection to server.
        let mut path = env::temp_dir();
        path.push("rzrtd.cli");

        let mut stream = match UnixStream::connect(path) {
            Ok(mut stream) => stream,
            Err(_) => return Err(CliError::ConnectError),
        };
        
        Ok(())
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
        fs::remove_file(TMP_FILE);
    }

    #[test]
    pub fn test_json_read() {
        let cli = Cli::new();
        let pathbuf = PathBuf::from(TMP_FILE);

        // No file written yet.
        let ret = cli.json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // Write non UTF text to file.
        let no_utf_txt = &[0xe9, 0x5a, 0xe9, 0x4a];
        write_tmp_file(no_utf_txt);
        let ret = cli.json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // UTF but not JSON.
        let utf_txt = "饂飩";
        write_tmp_file(utf_txt.as_bytes());
        let ret = cli.json_read(pathbuf.as_path());
        assert_eq!(ret, None);

        // Proper UTF JSON.
        let json_txt = "{\"noodle\":\"饂飩\"}";
        write_tmp_file(json_txt.as_bytes());
        let ret = cli.json_read(pathbuf.as_path());
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

        let ret = cli.init_cli_modes();
        let json = serde_json::from_str(&mode_json_str).unwrap();
        let ret = cli.build_mode(&json, None);
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