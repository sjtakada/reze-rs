//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Match Result and Flag.
//

use std::fmt;

// Match flag.
#[derive(PartialEq, PartialOrd, Eq, Ord, Copy, Clone)]
pub enum MatchFlag {
    Full,         // Fully matched.
    Partial,      // Partially matched, still valid.
    Incomplete,   // String incomplete, not valid for execution.
}

impl MatchFlag {
    pub fn to_string(&self) -> &str {
        match *self {
            MatchFlag::Full => "Match full",
            MatchFlag::Partial => "Match partial",
            MatchFlag::Incomplete => "Match incomplete",
        }
    }
}

impl fmt::Debug for MatchFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Match result:
//   If it succeeds, it will return either Full, Partial or incomplete, depending on the type of node.
//   If it fails, it will return position where the match fails.
#[derive(PartialEq, Copy, Clone)]
pub enum MatchResult {
    Failure(usize),
    Success(MatchFlag),
}

impl MatchResult {
    pub fn to_string(&self) -> String {
        match self {
            MatchResult::Failure(pos) => format!("Match failure at {}", pos),
            MatchResult::Success(flag) => {
                match flag {
                    MatchFlag::Full => "Fully matched".to_string(),
                    MatchFlag::Partial => "Partially matched".to_string(),
                    MatchFlag::Incomplete => "Incomplete match".to_string(),
                }
            }
        }
    }
}

impl fmt::Debug for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

