
use resp::{Value, Decoder};
use profile::sample::{ThreadData, MethodData};

pub fn resp_encode_thread_data(thread_data: &ThreadData) -> Vec<u8> {
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
        resp_encode_stacktrace(thread_data),
    ]);

    cmdValue.encode()
}


fn resp_encode_stacktrace(thread_data: &ThreadData) -> Value {
    let mut vec = vec![];
    for call_id in  &thread_data.stacktrace {
        vec.push(Value::Integer(call_id.clone()));
    }
    Value::Array(vec)
}

pub fn resp_encode_method_data(method_data: &MethodData) -> Vec<u8> {
    Value::Array(vec![
        Value::String("method".to_string()),
        Value::String("id".to_string()),
        Value::Integer(method_data.method_id),
        Value::String("name".to_string()),
        Value::String(method_data.full_name.clone()),
    ]).encode()
}