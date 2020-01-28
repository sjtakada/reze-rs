//
// ReZe.Rs - Router Daemon
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Core - Misc. utils.
//

use std::str::FromStr;

use common::error::*;
use common::method::Method;

/// Parse request string, and return method, path and request body.
pub fn request_parse(request: String) -> Result<(Method, String, Option<String>), CoreError> {
    let mut lines = request.lines();

    if let Some(req) = lines.next() {
        let mut words = req.split_ascii_whitespace();

        if let Some(method_str) = words.next() {
            if let Ok(method) = Method::from_str(method_str) {

                if let Some(path) = words.next() {
                    let mut body: Option<String> = None;

                    // Skip a blank line and get body if it is present.
                    if let Some(_) = lines.next() {
                        if let Some(b) =  lines.next() {
                            body = Some(b.to_string());
                        }
                    }

                    Ok((method, path.to_string(), body))
                } else {
                    Err(CoreError::RequestInvalid(req.to_string()))
                }
            } else {
                Err(CoreError::RequestInvalid(req.to_string()))
            }
        } else {
            Err(CoreError::RequestInvalid(req.to_string()))
        }
    } else {
        Err(CoreError::RequestInvalid("(no request line)".to_string()))
    }
}

