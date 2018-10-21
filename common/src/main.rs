extern crate common;

use common::access_list;

fn main() {
    println!("Hello, world!");

    let mut a = access_list::AccessList::new("100".to_string());
    a.add_rule(access_list::AclPermission::Permit, "1.1.1.1".to_string());
    a.add_rule(access_list::AclPermission::Deny, "2.2.2.2".to_string());

    a.show();

    a.add_rule(access_list::AclPermission::Deny, "3.3.3.3".to_string());
    a.show();
}

