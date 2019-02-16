//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Node.
//

use std::rc::Rc;

use super::tree;
use super::collate::*;


// CLI Node trait.
pub trait CliNode {
    // Return ClI token string ref.
    fn token(&self) -> &str;
    
    // Return match result and flag against input.
    fn collate(&self, input: &str) -> (MatchResult, MatchFlag);

    // Return help string ref.
    fn help(&self) -> &str;

    // Return defun token.
    fn defun_token(&self) -> &str;
}

// Common field for CliNode
pub struct CliNodeInner {
    // Node ID.
    id: String,

    // Defun token.
    defun_token: String,

    // Help string.
    help: String,

    // CLI token.
    cli_token: String,

    // Node vector is sorted.
    sorted: bool,

    // Hidden flag.
    hidden: bool,

    // Next candidate.
    next: Vec<Rc<CliNode>>,
}

impl CliNodeInner {
    pub fn new(id: String, defun_token: String, help: String, cli_token: String) -> CliNodeInner {
        CliNodeInner {
            id: id,
            defun_token: defun_token,
            help: help,
            cli_token: cli_token,
            sorted: false,
            hidden: false,
            next: Vec::new(),
        }
    }
}

pub struct CliNodeKeyword {

}

pub struct CliNodeRange {

}

pub struct CliNodeIPv4Prefix {

}

pub struct CliNodeIPv4Address {

}
    
pub struct CliNodeIPv6Prefix {

}

pub struct CliNodeIPv6Address {

}
    
pub struct CliNodeWord {

}

