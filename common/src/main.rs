extern crate common;

use common::acl::*;

fn main() {
    let mut a = AclBasic::new("100".to_string());
    a.add_rule(AclPerm::Permit, "1.1.1.1");
    a.add_rule(AclPerm::Deny, "2.2.2.2");

    a.show();

    a.add_rule(AclPerm::Deny, "3.3.3.3");
    a.add_rule(AclPerm::Deny, "any");
    a.add_rule(AclPerm::Deny, "hoge");
    a.add_rule(AclPerm::Deny, "1000");
    a.show();
}

