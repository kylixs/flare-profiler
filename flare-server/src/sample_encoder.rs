use std::mem;
use resp::{Value, Decoder};
use super::sample::ThreadData;
use super::utils::*;


pub fn resp_encode_thread_data(thread_data: &ThreadData) -> Value {
    Value::Array(vec![
        Value::String("thread".to_string()),
        Value::String("time".to_string()),
        Value::Integer(thread_data.sample_time),
        Value::String("id".to_string()),
        Value::Integer(thread_data.id),
        Value::String("name".to_string()),
        Value::String(thread_data.name.clone()),
        Value::String("cpu_time".to_string()),
        Value::Integer(thread_data.cpu_time),
        Value::String("cpu_time_delta".to_string()),
        Value::Integer(thread_data.cpu_time_delta),
        Value::String("state".to_string()),
        Value::String(thread_data.state.clone()),
        Value::String("stacktrace".to_string()),
        resp_encode_stacktrace(thread_data),
    ])
}


fn resp_encode_stacktrace(thread_data: &ThreadData) -> Value {
//    let mut vec = Vec::with_capacity(thread_data.stacktrace.len());
//    for call_id in &thread_data.stacktrace {
//        vec.push(Value::Integer(*call_id));
//    }
//    Value::Array(vec)

    let vec64 = thread_data.stacktrace.clone();

    // I copy-pasted this code from StackOverflow without reading the answer
    // surrounding it that told me to write a comment explaining why this code
    // is actually safe for my own use case.
    let vec8 = unsafe {
        let ratio = mem::size_of::<i64>() / mem::size_of::<u8>();

        let length = vec64.len() * ratio;
        let capacity = vec64.capacity() * ratio;
        let ptr = vec64.as_ptr() as *mut u8;

        // Don't run the destructor for vec32
        mem::forget(vec64);

        // Construct new Vec
        Vec::from_raw_parts(ptr, length, capacity)
    };
    Value::BufBulk(vec8)
}


pub fn resp_decode_thread_data(data_vec: &Vec<resp::Value>) -> ThreadData {
//    let sample_time = get_resp_property_as_int(data_vec, "time", 1, 0);
//    let thread_id = get_resp_property_as_int(data_vec, "id", 1, 0);
//    let cpu_time = get_resp_property_as_int(data_vec, "cpu_time", 1, 0);
//    let cpu_time_delta = get_resp_property_as_int(data_vec, "cpu_time_delta", 1, 0);
//    let name = get_resp_property_as_str(data_vec, "name", 1, "");
//    let state = get_resp_property_as_str(data_vec, "state", 1, "");
//    let mut stacktrace = vec![];
//    let data = get_resp_property(data_vec, "stacktrace", 1);
//    if let Some(resp::Value::BufBulk(vec)) = data {
//        stacktrace = convert_to_vec64(vec.clone());
//    }

    let mut stacktrace = vec![];
    let mut sample_time= 0;
    let mut thread_id= 0;
    let mut cpu_time= 0;
    let mut cpu_time_delta= 0;
    let mut name= "";
    let mut state= "";
    for x in (1 as usize..data_vec.len()).step_by(2) {
        if let resp::Value::String(key) = &data_vec[x] {
            match key.as_ref() {
                "time" => {
                    parse_as_int(&data_vec[x+1], 0);
                }
                "thread_id" => {
                    parse_as_int(&data_vec[x+1], 0);
                }
                "cpu_time" => {
                    parse_as_int(&data_vec[x+1], 0);
                }
                "cpu_time_delta" => {
                    parse_as_int(&data_vec[x+1], 0);
                }
                "name" => {
                    parse_as_string(&data_vec[x+1], "");
                }
                "state" => {
                    parse_as_string(&data_vec[x+1], "");
                }
                "stacktrace" => {
                    if let resp::Value::BufBulk(vec) = &data_vec[x+1] {
                        stacktrace = convert_to_vec64(vec.clone());
                    }
                }
                _ => {}
            }
        }
    }

    ThreadData {
        id: thread_id,
        name: name.to_string(),
        priority: 0,
        daemon: false,
        state: state.to_string(),
        cpu_time: cpu_time,
        cpu_time_delta: cpu_time_delta,
        sample_time: sample_time,
        stacktrace: stacktrace,
        duration: 0,
        self_duration: 0,
        self_cpu_time: 0
    }
}

fn parse_as_int(resp_val: &Value, default_value: i64) -> i64 {
    if let resp::Value::Integer(x) = resp_val {
        return *x;
    }
    return default_value;
}

fn parse_as_string<'a >(resp_val: &'a Value, default_value: &'a str) -> &'a str {
    if let resp::Value::String(x) = resp_val {
        return x;
    }
    return default_value;
}

pub fn convert_to_vec64(vec8: Vec<u8>) -> Vec<i64> {
    let vec64 = unsafe {
        let ratio = mem::size_of::<i64>() / mem::size_of::<u8>();

        let length = vec8.len() / ratio;
        let capacity = vec8.capacity() / ratio;
        let ptr = vec8.as_ptr() as *mut i64;

        // Don't run the destructor for vec32
        mem::forget(vec8);

        // Construct new Vec
        Vec::from_raw_parts(ptr, length, capacity)
    };
    vec64
}