//
// ACL: Access Control List
//
// Example:
//   access-list 100 permit 1.1.1.1
//   access-list 100 deny 2.2.2.2
//   access-list 100 permit any
//

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::str::FromStr;
use std::net::Ipv4Addr;
use std::net::AddrParseError;

// Type definitions.

// (permit|deny)
pub enum AclPerm {
    Permit,
    Deny
}

// (A.B.C.D|any)
pub enum AclRuleAddr {
    Addr(Ipv4Addr),
    Any
}

// Basic ACL rule: (permit|deny) (A.B.C.D|any)
pub struct AclRule {
    perm: AclPerm,
    addr: AclRuleAddr,
}

// List of ACL rules.
pub struct Acl {
    pub name: String,
    rules: Vec<AclRule>,
}

// Collections of ACLs.
pub struct AclCollection {
    acls: HashMap<String, Acl>
}

// Implementations.

// AclPerm
impl AclPerm {
    pub fn to_string(&self) -> &str {
        match *self {
            AclPerm::Permit => "permit",
            AclPerm::Deny => "deny"
        }
    }
}

// AclRuleAddr
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

// AclRule
impl AclRule {
    pub fn new(perm: AclPerm, addr: AclRuleAddr) -> Self {
        AclRule { perm, addr }
    }
}

// Acl
impl Acl {
    pub fn new(name: String) -> Self {
        Acl{ name: name, rules: Vec::new() }
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

impl AclCollection {
    pub fn new() -> Self {
        AclCollection { acls: HashMap::new() }
    }

    pub fn get_mut(&mut self, name: &str) -> &mut Acl {
        match self.acls.entry(name.to_string()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Acl::new(String::from(name)))
        }
    }

//    pub fn get(&mut self, name: &str) -> &Acl {
//        match self.acls.entry(name.to_string()) {
//            Entry::Occupied(o) => o.get(),
//            Entry::Vacant(v) => ()
//        }
//    }

    pub fn show(&self, name: &str) {
    }
}

// Tests.
