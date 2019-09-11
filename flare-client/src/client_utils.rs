use std::collections::HashMap;
use websocket::OwnedMessage;
use serde::Serialize;
use FlareResponse;
use serde_json::{Number,json};
use std::io::ErrorKind;
use std::io;

pub fn get_resp_property<'a>(data_vec: &'a Vec<resp::Value>, key: &str, start: i32) -> Option<&'a resp::Value> {
    for x in (start as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(name) = &data_vec[x] {
            if name == key {
                return Some(&data_vec[x+1]);
            }
        }
    }
    None
}

pub fn get_resp_property_as_int(data_vec: &Vec<resp::Value>, key: &str, start: i32, default_value: i64) -> i64 {
    for x in (start as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(name) = &data_vec[x] {
            if name == key {
                if let resp::Value::Integer(x) = &data_vec[x+1] {
                    return *x
                }
            }
        }
    }
    default_value
}

pub fn get_resp_property_as_str<'a>(data_vec: &'a Vec<resp::Value>, key: &str, start: i32, default_value: &'a str) -> &'a str {
    for x in (start as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(name) = &data_vec[x] {
            if name == key {
                if let resp::Value::String(x) = &data_vec[x+1] {
                    return x
                }
            }
        }
    }
    default_value
}

pub fn parse_resp_properties<'a>(data_vec: &'a Vec<resp::Value>, start: i32) -> HashMap<&'a String, &'a resp::Value> {
    let mut map = HashMap::new();
    for x in (start as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(name) = &data_vec[x] {
            map.insert(name, &data_vec[x+1]);
        }
    }
    map
}


pub fn wrap_response<T: ?Sized>(cmd: &str, value: &T) -> OwnedMessage
    where
        T: Serialize,
{
    let response = FlareResponse {
        result: "success".to_string(),
        cmd: cmd.to_string(),
        data: Box::new(value)
    };
    OwnedMessage::Text(serde_json::to_string(&response).unwrap())
}

pub fn wrap_error_response(cmd: &str, message: &str) -> OwnedMessage {
    let response = FlareResponse {
        result: "failure".to_string(),
        cmd: cmd.to_string(),
        data: Box::new(json!({ "message": message }))
    };
    OwnedMessage::Text(serde_json::to_string(&response).unwrap())
}

pub fn get_option_required_as_str<'a>(options: &'a serde_json::Map<String, serde_json::Value>, key: &str) -> io::Result<&'a str> {
    match options.get(key) {
        Some(val) => {
            match val.as_str() {
                Some(s) => Ok(s),
                None => {
                    Err(io::Error::new(ErrorKind::InvalidInput, format!("option value is not a string '{}'", key)))
                },
            }
        },
        None => {
            Err(io::Error::new(ErrorKind::InvalidInput, format!("missing option '{}'", key)))
        }
    }
}

pub fn get_option_as_int(request: &serde_json::Value, key: &str) -> Option<Number> {
    if let serde_json::Value::Object(options) = &request["options"] {
        if let serde_json::Value::Number(s) = &options[key] {
            return Some(s.clone());
        }
    }
    return None;
}

pub fn new_error(kind: ErrorKind, msg: &str) -> io::Result<()> {
    Err(io::Error::new(kind, msg))
}

pub fn new_invalid_input_error(msg: &str) -> io::Result<()> {
    Err(io::Error::new(ErrorKind::InvalidInput, msg))
}