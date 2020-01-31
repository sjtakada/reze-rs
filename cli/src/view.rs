//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// View template.
//

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::io::Write;

use super::error::*;

/// Cli View
pub struct CliView {

    /// Templates.
    templates: HashMap<String, &'static Fn(&serde_json::Value) -> Result<(), CliError>>,
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
    pub fn register(&mut self, source: &str, func: &'static Fn(&serde_json::Value) -> Result<(), CliError>) {
        self.templates.insert(source.to_string(), func);
    }

    /// Initialize.
    pub fn init(&mut self) {
        self.register("dummy", &CliView::dummy);
        self.register("show_ip_route", &CliView::show_ip_route);
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
            .expect("Exec failed");

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(value.to_string().as_bytes());
        } else {
            println!("Failed to open stdin for child process");
//            Err(CliError::);
        }

        Ok(())
    }

    /// Dummy.
    pub fn dummy(value: &serde_json::Value) -> Result<(), CliError> {
        println!("dummy");
        Ok(())
    }

    /// "show ip route"
    pub fn show_ip_route(value: &serde_json::Value) -> Result<(), CliError> {
        println!("\
Codes: C - connected, S - static, R - RIP, B - BGP
       D - EIGRP, EX - EIGRP external, O - OSPF, IA - OSPF inter area 
       N1 - OSPF NSSA external type 1, N2 - OSPF NSSA external type 2
       E1 - OSPF external type 1, E2 - OSPF external type 2
       i - IS-IS, su - IS-IS summary, L1 - IS-IS level-1, L2 - IS-IS level-2
       ia - IS-IS inter area, * - candidate default
");

        if value.is_array() {
            for r in value.as_array().unwrap() {
                let prefix = &r["prefix"];
                let entry = &r["entry"];

                if prefix.is_string() && entry.is_object() {
                    let prefix = prefix.as_str().unwrap();
                    let entry = entry.as_object().unwrap();

                    if !entry["type"].is_string() {
                        continue;
                    }
                    let rib_type = entry["type"].as_str().unwrap();

                    if !entry["distance"].is_number() {
                        continue;
                    }
                    let distance = entry["distance"].as_u64().unwrap();

                    if !entry["nexthops"].is_array() {
                        continue;
                    }
                    let nexthops = entry["nexthops"].as_array().unwrap();

                    for nh in nexthops {
                        if nh["address"].is_string() {
                            println!("{} {} {}", rib_type, prefix, nh["address"].as_str().unwrap());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
