use std::collections::HashMap;
use websocket::OwnedMessage;
use serde::Serialize;
use FlareResponse;

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