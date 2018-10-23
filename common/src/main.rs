extern crate common;

use common::acl::*;
use std::str::FromStr;
use std::net::Ipv4Addr;

fn main() {
    let mut ac = AclCollection::new();
//    let mut &acl = ac.get("100");

    ac.get_mut("100").add_rule(AclPerm::Permit, "1.1.1.1");
    ac.get_mut("100").add_rule(AclPerm::Deny, "2.2.2.2");

    ac.show("100");

    ac.get_mut("100").add_rule(AclPerm::Deny, "3.3.3.3");
    ac.get_mut("100").add_rule(AclPerm::Deny, "any");
    ac.get_mut("100").add_rule(AclPerm::Deny, "hoge");
    ac.get_mut("100").add_rule(AclPerm::Deny, "1000");
    //ac.get_mut("100").show();
    ac.show("100");

    println!("--");
    let a = Ipv4Addr::from_str("1.1.1.1").unwrap();
    println!("acl check {} {}", "1.1.1.1", ac.check("100", &a).to_string());

    let a = Ipv4Addr::from_str("2.2.2.2").unwrap();
    println!("acl check {} {}", "2.2.2.2", ac.check("100", &a).to_string());

    let a = Ipv4Addr::from_str("3.3.3.3").unwrap();
    println!("acl check {} {}", "3.3.3.3", ac.check("100", &a).to_string());

    let a = Ipv4Addr::from_str("4.4.4.4").unwrap();
    println!("acl check {} {}", "4.4.4.4", ac.check("100", &a).to_string());
}

