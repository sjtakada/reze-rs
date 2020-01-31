//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// View template.
//

use std::io;
use std::io::Write;
use std::collections::HashMap;
use std::process::{Command, Stdio};

use super::error::*;

/// Cli View
pub struct CliView {

    /// Templates.
    templates: HashMap<String, &'static dyn Fn(&serde_json::Value) -> Result<(), CliError>>,
}

/// Cli View implementation.
impl CliView {

    /// Constructor.
    pub fn new() -> CliView {
        CliView {
            templates: HashMap::new(),
        }
    }

    /// Register view tempate function.
    pub fn register(&mut self, name: &str, func: &'static dyn Fn(&serde_json::Value) -> Result<(), CliError>) {
        self.templates.insert(name.to_string(), func);
    }

    /// Initialize.
    pub fn init(&mut self) {
        self.register("dummy", &CliView::dummy);
    }

    /// Call template function.
    pub fn call(&self, func: &str, value: &serde_json::Value) -> Result<(), CliError> {
        match self.templates.get(func) {
            Some(template) => template(value),
            None => {
                println!("No template found");
                Ok(())
            }
        }
    }

    /// Execute extrenal template engine.
    pub fn exec(&self, path: &str, params: &str, value: &serde_json::Value) -> Result<(), CliError> {
        let mut child = Command::new(path)
            .arg(params)
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to execute a child");

        if let Some(stdin) = child.stdin.as_mut() {
            if let Err(err) = stdin.write_all(value.to_string().as_bytes()) {
                println!("Failed to write to child process {:?}", err);
                return Err(CliError::ChildProcessError)
            }
        } else {
            println!("Failed to write to child process");
            return Err(CliError::ChildProcessError)
        }

        let output = child.wait_with_output()
            .expect("Failed to wait on child");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        Ok(())
    }

    /// Dummy.
    pub fn dummy(_value: &serde_json::Value) -> Result<(), CliError> {
        println!("dummy");
        Ok(())
    }
}
