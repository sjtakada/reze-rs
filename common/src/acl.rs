//
// access-list 100 permit 1.1.1.1
// access-list 100 deny 2.2.2.2
// access-list 100 permit any
//
pub enum AclPerm {
    Permit,
    Deny
}

pub enum AclRuleAddr {
    Addr(String),
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
    pub fn to_string(&self) -> String {
        match self {
            AclRuleAddr::Addr(s) => s.clone(),
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
        let a = match addr {
            "any" => AclRuleAddr::Any,
            _ => AclRuleAddr::Addr(String::from(addr))
        };

        self.rules.push(AclRule::new(perm, a));
    }

    pub fn show(&self) {
        for r in &self.rules {
            println!("access-list {} {} {}",
                     self.name, r.perm.to_string(), r.addr.to_string());
        }
    }
}

