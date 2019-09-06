use resp::{Value, Decoder};
use super::sampler_client::ThreadData;

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
    let mut vec = vec![];
    for call_id in  &thread_data.stacktrace {
        vec.push(Value::Integer(call_id.clone()));
    }
    Value::Array(vec)
}
