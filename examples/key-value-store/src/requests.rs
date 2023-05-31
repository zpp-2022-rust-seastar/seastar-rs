use super::server::{ConnectionError, ConnectionResult};
use regex::Regex;

pub enum Request {
    Store(StoreRequest),
    Load(LoadRequest),
}

pub struct StoreRequest {
    pub key: String,
    pub value: String,
}

impl StoreRequest {
    pub fn new(key: String, value: String) -> Self {
        StoreRequest { key, value }
    }
}

pub struct LoadRequest {
    pub key: String,
}

impl LoadRequest {
    pub fn new(key: String) -> Self {
        LoadRequest { key }
    }
}

static STORE: &str = "STORE$";
static LOAD: &str = "LOAD$";

fn match_regex(message: &str, pattern: &str) -> ConnectionResult<bool> {
    match Regex::new(pattern) {
        Ok(regex) => Ok(regex.is_match(message)),
        Err(_) => Err(ConnectionError),
    }
}

// Returns true if there exists a prefix of a message parameter
// that is a correct STORE request.
fn is_store_request(message: &str) -> ConnectionResult<bool> {
    match_regex(message, r#"^STORE\$[a-z]*\$[a-z]*\n"#)
}

// Returns true if there exists a prefix of a message parameter
// that is a correct LOAD request.
fn is_load_request(message: &str) -> ConnectionResult<bool> {
    match_regex(message, r#"^LOAD\$[a-z]*\n"#)
}

// Returns true if message could become a correct STORE request
// after appending more chars.
fn could_become_store_request(message: &str) -> ConnectionResult<bool> {
    if message.len() <= STORE.len() {
        return Ok(message == &STORE[..message.len()]);
    }

    Ok(match_regex(message, r"^STORE\$[a-z]*$")?
        || match_regex(message, r"^STORE\$[a-z]*\$[a-z]*$")?)
}

// Returns true if message could become a correct LOAD request
// after appending more chars.
fn could_become_load_request(message: &str) -> ConnectionResult<bool> {
    if message.len() <= LOAD.len() {
        return Ok(message == &LOAD[..message.len()]);
    }

    match_regex(message, r"^LOAD\$[a-z]*$")
}

// Splits a message with a prefix that is a correct STORE request
// from STORE$key$value\nrest to (key, value, rest).
fn split_store_request(message: &str) -> (String, String, String) {
    let message = &message[STORE.len()..];
    let dollar = message.find('$').unwrap();
    let newline = message.find('\n').unwrap();
    let key = message[..dollar].to_string();
    let value = message[dollar + 1..newline].to_string();
    let rest = message[newline + 1..].to_string();
    (key, value, rest)
}

// Splits a message with a prefix that is a correct LOAD request
// from LOAD$key\nrest to (key, rest).
fn split_load_request(message: &str) -> (String, String) {
    let newline = message.find('\n').unwrap();
    let key = message[LOAD.len()..newline].to_string();
    let rest = message[newline + 1..].to_string();
    (key, rest)
}

// If a message contains a prefix that is a correct request, returns
// Some(request). If the message is incorrect, returns ConnectionError.
// Otherwise, returns None. Removes the request part from the message.
pub fn try_parse_request(message: &mut String) -> ConnectionResult<Option<Request>> {
    if is_store_request(message)? {
        let (key, value, rest) = split_store_request(message);
        *message = rest;
        Ok(Some(Request::Store(StoreRequest::new(key, value))))
    } else if is_load_request(message)? {
        let (key, rest) = split_load_request(message);
        *message = rest;
        Ok(Some(Request::Load(LoadRequest::new(key))))
    } else if could_become_store_request(message)? || could_become_load_request(message)? {
        Ok(None)
    } else {
        Err(ConnectionError)
    }
}
