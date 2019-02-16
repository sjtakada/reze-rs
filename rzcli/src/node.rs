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
    // Return inner.
    fn inner(&self) -> &CliNodeInner;

    // Return ClI token string ref.
    fn token(&self) -> &str;
    
    // Return match result and flag against input.
    fn collate(&self, input: &str) -> (MatchResult, MatchFlag);

    // Return help string ref.
    //fn help(&self) -> &str;

    // Return defun token.
    //fn defun_token(&self) -> &str;
}

// Common field for CliNode
pub struct CliNodeInner {
    // Node ID.
    id: String,

    // Defun token.
    defun: String,

    // Help string.
    help: String,

    // CLI token.
    token: String,

    // Node vector is sorted.
    sorted: bool,

    // Hidden flag.
    hidden: bool,

    // Next candidate.
    next: Vec<Rc<CliNode>>,
}

impl CliNodeInner {
    pub fn new(id: String, defun: String, help: String, token: String) -> CliNodeInner {
        CliNodeInner {
            id: id,
            defun: defun,
            help: help,
            token: token,
            sorted: false,
            hidden: false,
            next: Vec::new(),
        }
    }

    fn help(&self) -> &str {
        &self.help
    }

    fn defun(&self) -> &str {
        &self.defun
    }
}

pub struct CliNodeKeyword {
    inner: CliNodeInner,
}

impl CliNodeKeyword {
    pub fn new(id: String, defun: String, help: String, token: String) -> CliNodeKeyword {
        CliNodeKeyword {
            inner: CliNodeInner::new(id, defun, help, token)
        }
    }
}

impl CliNode for CliNodeKeyword {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
    }
    
    fn collate(&self, input: &str) -> (MatchResult, MatchFlag) {
        if input == self.token() {
            return (MatchResult::Success, MatchFlag::Full)
        }

        let t = &self.token()[0..input.len()];
        if input == t {
            return (MatchResult::Success, MatchFlag::Partial)
        }

        (MatchResult::Failure, MatchFlag::None)
    }

}

pub struct CliNodeRange {
    inner: CliNodeInner,
}

pub struct CliNodeIPv4Prefix {
    inner: CliNodeInner,
}

pub struct CliNodeIPv4Address {
    inner: CliNodeInner,
}
    
pub struct CliNodeIPv6Prefix {
    inner: CliNodeInner,
}

pub struct CliNodeIPv6Address {
    inner: CliNodeInner,
}
    
pub struct CliNodeWord {
    inner: CliNodeInner,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_node_keyword() {
        let node = CliNodeKeyword::new(String::from("show"), String::from("show"),
                                       String::from("help"), String::from("show"));
        let (result, flag) = node.collate("show");
        assert_eq!(result, MatchResult::Success);
        assert_eq!(flag, MatchFlag::Full);

        let (result, flag) = node.collate("sho");
        assert_eq!(result, MatchResult::Success);
        assert_eq!(flag, MatchFlag::Partial);

        let (result, flag) = node.collate("shop");
        assert_eq!(result, MatchResult::Failure);
        assert_eq!(flag, MatchFlag::None);
    }
}

