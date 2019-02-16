//
// ReZe.Rs - ReZe CLI
//   Copyright (C) 2018,2019 Toshiaki Takada
//
// CLI Match Result and Flag.
//

// Match result.
pub enum MatchResult {
    Failure,
    Success,
}

// Match flag.
pub enum MatchFlag {
    Full,         // Fully matched.
    Partial,      // Partially matched, still valid.
    Incomplete,   // String incomplete, not valid for execution.
    None,         // Not matched.
}
