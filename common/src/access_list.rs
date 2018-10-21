//
// access-list 100 permit 1.1.1.1
// access-list 100 deny 2.2.2.2
// access-list 100 permit any
//
pub enum AclPermission {
    Permit,
    Deny
}

pub struct AccessListRule {
    perm: AclPermission,
    addr: String,
}

pub struct AccessList {
    pub name: String,
    rules: Vec<AccessListRule>,
}

// 
impl AclPermission {
    pub fn to_string(&self) -> &str {
        match *self {
            AclPermission::Permit => "permit",
            AclPermission::Deny => "deny"
        }
    }
}

//
impl AccessListRule {
    pub fn new(perm: AclPermission, addr: String) -> AccessListRule {
        AccessListRule { perm, addr }
    }
}

//
impl AccessList {
    pub fn new(name: String) -> AccessList {
        AccessList{ name: name, rules: Vec::new() }
    }

    pub fn add_rule(&mut self, perm:  AclPermission, addr: String) {
        let rule = AccessListRule::new(perm, addr);
        self.rules.push(rule);
    }

    pub fn show(&self) {
        for r in &self.rules {
            println!("access-list {} {} {}",
                     self.name, r.perm.to_string(), r.addr);
        }
    }
}

