//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Match Result and Flag.
//

use std::fmt;

// Match flag.
#[derive(PartialEq)]
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

// Match result.
#[derive(PartialEq)]
pub enum MatchResult {
    Failure,
    Success(MatchFlag),
}

impl MatchResult {
    pub fn to_string(&self) -> &str {
        match self {
            MatchResult::Failure => "Match failure",
            MatchResult::Success(flag) => {
                match flag {
                    MatchFlag::Full => "Fully matched",
                    MatchFlag::Partial => "Partially matched",
                    MatchFlag::Incomplete => "Incomplete match",
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

