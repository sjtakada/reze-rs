//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Global CLI configuration.
//

use std::io::BufReader;
use std::io::Read;
use std::fs::File;
use std::path::Path;

use serde_json;

// Read and return JSON, if it fails, return None.
pub fn json_read(path: &Path) -> Option<serde_json::Value> {
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

