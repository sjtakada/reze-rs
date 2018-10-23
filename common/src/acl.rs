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
use std::fmt;

// Type definitions.

// (permit|deny)
#[derive(PartialEq)]
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
    rules: Vec<AclRule>,
}

// Collections of ACLs.
pub struct AclCollection {
    acls: HashMap<String, Acl>,
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

impl fmt::Debug for AclPerm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
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
    pub fn new() -> Self {
        Acl{ rules: Vec::new() }
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

    pub fn check(&self, addr: &Ipv4Addr) -> &AclPerm {
        for r in &self.rules {
            match r.addr {
                AclRuleAddr::Addr(a) => {
                    if a == *addr {
                        return &r.perm
                    }
                }
                AclRuleAddr::Any => {
                    return &r.perm
                }
            }
        }
        &AclPerm::Deny
    }
}

// AclCollection
impl AclCollection {
    pub fn new() -> Self {
        AclCollection { acls: HashMap::new() }
    }

    pub fn get_mut(&mut self, name: &str) -> &mut Acl {
        match self.acls.entry(name.to_string()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Acl::new())
        }
    }

    pub fn get(&self, name: &str) -> Option<&Acl> {
        self.acls.get(name)
    }

    pub fn check(&self, name: &str, addr: &Ipv4Addr) -> &AclPerm {
        match self.get(name) {
            Some(r) => &r.check(addr),
            None => &AclPerm::Deny
        }
    }

    pub fn show(&self, name: &str) {
        match self.acls.get(name) {
            Some(acl) => {
                for r in &acl.rules {
                    println!("access-list {} {} {}",
                             name, r.perm.to_string(), r.addr.to_string());
                }
            }
            None => {
            }
        }
    }
}

// Tests.

#[test]
pub fn test_acl () {
    let mut ac = AclCollection::new();

    ac.get_mut("100").add_rule(AclPerm::Permit, "1.1.1.1");
    ac.get_mut("100").add_rule(AclPerm::Deny, "2.2.2.2");
    ac.get_mut("100").add_rule(AclPerm::Deny, "3.3.3.3");
    ac.get_mut("100").add_rule(AclPerm::Deny, "any");

    // XXX unwrap
    let a = Ipv4Addr::from_str("1.1.1.1").unwrap();
    assert_eq!(&AclPerm::Permit, ac.check("100", &a));

    let a = Ipv4Addr::from_str("2.2.2.2").unwrap();
    assert_eq!(&AclPerm::Deny, ac.check("100", &a));

    let a = Ipv4Addr::from_str("3.3.3.3").unwrap();
    assert_eq!(&AclPerm::Deny, ac.check("100", &a));

    let a = Ipv4Addr::from_str("4.4.4.4").unwrap();
    assert_eq!(&AclPerm::Deny, ac.check("100", &a));

    assert_eq!(&AclPerm::Deny, ac.check("200", &a));
}
