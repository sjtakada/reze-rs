extern crate common;

use common::acl::*;

fn main() {
    let mut ac = AclCollection::new();
//    let mut &acl = ac.get("100");

    ac.get_mut("100").add_rule(AclPerm::Permit, "1.1.1.1");
    ac.get_mut("100").add_rule(AclPerm::Deny, "2.2.2.2");

    ac.get_mut("100").show();

    ac.get_mut("100").add_rule(AclPerm::Deny, "3.3.3.3");
    ac.get_mut("100").add_rule(AclPerm::Deny, "any");
    ac.get_mut("100").add_rule(AclPerm::Deny, "hoge");
    ac.get_mut("100").add_rule(AclPerm::Deny, "1000");
    ac.get_mut("100").show();
}

