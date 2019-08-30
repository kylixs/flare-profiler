use std::collections::HashMap;

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

pub fn parse_resp_properties<'a>(data_vec: &'a Vec<resp::Value>, start: i32) -> HashMap<&'a String, &'a resp::Value> {
    let mut map = HashMap::new();
    for x in (start as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(name) = &data_vec[x] {
            map.insert(name, &data_vec[x+1]);
        }
    }
    map
}