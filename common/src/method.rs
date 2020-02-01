//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Method
//

use std::fmt;
use std::str::FromStr;

use super::error::CoreError;

/// Method: equivalent to HTTP Method.
#[derive(Copy, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// FromStr.
impl FromStr for Method {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let method = s.to_lowercase();

        match method.as_ref() {
            "get" => Ok(Method::Get),
            "post" => Ok(Method::Post),
            "put" => Ok(Method::Put),
            "delete" => Ok(Method::Delete),
            "patch" => Ok(Method::Patch),
            _ => Err(CoreError::ParseMethod),
        }
    }
}

/// Display.
impl fmt::Display for Method {

    /// Format method.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        };

        write!(f, "{}", s)
    }
}

