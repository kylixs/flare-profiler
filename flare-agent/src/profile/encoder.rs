
use resp::{Value, Decoder};
use profile::sample::ThreadData;

pub fn encode_sample_data_result(thread_data: &ThreadData) -> Vec<u8> {
    let cmdValue = Value::Array(vec![
        Value::String("thread".to_string()),
        Value::String("time".to_string()),
        Value::Integer(thread_data.sample_time),
        Value::String("id".to_string()),
        Value::Integer(thread_data.id),
        Value::String("name".to_string()),
        Value::String(thread_data.name.clone()),
        Value::String("cpu_time".to_string()),
        Value::Integer(thread_data.cpu_time),
        Value::String("state".to_string()),
        Value::String(thread_data.state.clone()),
        Value::String("stacktrace".to_string()),
        encode_stacktrace(thread_data),
    ]);

    cmdValue.encode()
}


fn encode_stacktrace(thread_data: &ThreadData) -> Value {
    let mut vec = vec![];
    for call_name in  &thread_data.stacktrace {
        vec.push(Value::String(call_name.clone()));
    }
    Value::Array(vec)
}