//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Match Result and Flag.
//

use std::fmt;

// Match result.
#[derive(PartialEq)]
pub enum MatchResult {
    Failure,
    Success,
}

impl MatchResult {
    pub fn to_string(&self) -> &str {
        match *self {
            MatchResult::Success => "Match success",
            MatchResult::Failure => "Match failure",
        }
    }
}

impl fmt::Debug for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Match flag.
#[derive(PartialEq)]
pub enum MatchFlag {
    Full,         // Fully matched.
    Partial,      // Partially matched, still valid.
    Incomplete,   // String incomplete, not valid for execution.
    None,         // Not matched.
}

impl MatchFlag {
    pub fn to_string(&self) -> &str {
        match *self {
            MatchFlag::Full => "Match full",
            MatchFlag::Partial => "Match partial",
            MatchFlag::Incomplete => "Match incomplete",
            MatchFlag::None => "Match none",
        }
    }
}

impl fmt::Debug for MatchFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
