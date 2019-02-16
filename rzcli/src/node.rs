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
    fn collate(&self, input: &str) -> MatchResult;

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
    pub fn new(id: &str, defun: &str, help: &str, token: &str) -> CliNodeInner {
        CliNodeInner {
            id: String::from(id),
            defun: String::from(defun),
            help: String::from(help),
            token: String::from(token),
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

// CLI keyword node
//   static literal
pub struct CliNodeKeyword {
    inner: CliNodeInner,
}

impl CliNodeKeyword {
    pub fn new(id: &str, defun: &str, help: &str, token: &str) -> CliNodeKeyword {
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
    
    fn collate(&self, input: &str) -> MatchResult {
        if input == self.token() {
            return MatchResult::Success(MatchFlag::Full)
        }

        let t = &self.token()[0..input.len()];
        if input == t {
            return MatchResult::Success(MatchFlag::Partial)
        }

        MatchResult::Failure
    }

}

// CLI range node
//   integer range to match numeric input.
pub struct CliNodeRange {
    inner: CliNodeInner,
    min: i64,
    max: i64,
}

impl CliNodeRange {
    pub fn new(id: &str, defun: &str, help: &str,
               min: i64, max: i64) -> CliNodeRange {
        let token = format!("<{}-{}>", min, max);

        CliNodeRange {
            inner: CliNodeInner::new(id, defun, help, &token),
            min: min,
            max: max,
        }
    }
}

impl CliNode for CliNodeRange {
    fn inner(&self) -> &CliNodeInner {
        &self.inner
    }

    fn token(&self) -> &str {
        &self.inner.token
    }
    
    fn collate(&self, input: &str) -> MatchResult {
        let num = match input.parse::<i64>() {
            Ok(num) => num,
            Err(_err) => return MatchResult::Failure,
        };

        if num >= self.min && num <= self.max {
            return MatchResult::Success(MatchFlag::Full)
        }

        MatchResult::Failure
    }
}

//
pub struct CliNodeIPv4Prefix {
    inner: CliNodeInner,
}

//
pub struct CliNodeIPv4Address {
    inner: CliNodeInner,
}
    
//
pub struct CliNodeIPv6Prefix {
    inner: CliNodeInner,
}

//
pub struct CliNodeIPv6Address {
    inner: CliNodeInner,
}
    
//
pub struct CliNodeWord {
    inner: CliNodeInner,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_node_keyword() {
        let node = CliNodeKeyword::new("show", "show", "help", "show");

        let result = node.collate("show");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("sho");
        assert_eq!(result, MatchResult::Success(MatchFlag::Partial));

        let result = node.collate("shop");
        assert_eq!(result, MatchResult::Failure);
    }

    #[test]
    pub fn test_node_range() {
        let node = CliNodeRange::new("range", "RANGE", "help", 100i64, 9999i64);

        assert_eq!(node.token(), "<100-9999>");

        let result = node.collate("100");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("99");
        assert_eq!(result, MatchResult::Failure);

        let result = node.collate("9999");
        assert_eq!(result, MatchResult::Success(MatchFlag::Full));

        let result = node.collate("10000");
        assert_eq!(result, MatchResult::Failure);
    }
}

