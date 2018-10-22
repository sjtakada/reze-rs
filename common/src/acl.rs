//
// ACL: Access Control List
//
// access-list 100 permit 1.1.1.1
// access-list 100 deny 2.2.2.2
// access-list 100 permit any
//

use std::str::FromStr;
use std::net::Ipv4Addr;
use std::net::AddrParseError;

pub enum AclPerm {
    Permit,
    Deny
}

pub enum AclRuleAddr {
    Addr(Ipv4Addr),
    Any
}

pub struct AclRule {
    perm: AclPerm,
    addr: AclRuleAddr,
}

pub struct AclBasic {
    pub name: String,
    rules: Vec<AclRule>,
}

// 
impl AclPerm {
    pub fn to_string(&self) -> &str {
        match *self {
            AclPerm::Permit => "permit",
            AclPerm::Deny => "deny"
        }
    }
}

//
impl AclRuleAddr {
    pub fn from_str(s: &str) -> Result<AclRuleAddr, AddrParseError> {
        match s {
            "any" => Ok(AclRuleAddr::Any),
            addr => {
                match Ipv4Addr::from_str(addr) {
                    Ok(addr) => Ok(AclRuleAddr::Addr(addr)),
                    Err(e) => Err(e)
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AclRuleAddr::Addr(s) => s.to_string(),
            AclRuleAddr::Any => "any".to_string()
        }
    }
}

//
impl AclRule {
    pub fn new(perm: AclPerm, addr: AclRuleAddr) -> Self {
        AclRule { perm, addr }
    }
}

//
impl AclBasic {
    pub fn new(name: String) -> Self {
        AclBasic{ name: name, rules: Vec::new() }
    }

    pub fn add_rule(&mut self, perm: AclPerm, addr: &str) {
        let a = AclRuleAddr::from_str(addr);
        match a {
            Ok(addr) => {
                self.rules.push(AclRule::new(perm, addr));
            },
            Err(e) => {
            }
        }
    }

    pub fn show(&self) {
        for r in &self.rules {
            println!("access-list {} {} {}",
                     self.name, r.perm.to_string(), r.addr.to_string());
        }
    }
}

