#![feature(test)]

extern crate test;
extern crate resp;

use test::Bencher;
use std::io::BufReader;
use resp::{Value, Decoder};

fn prepare_values() -> Value {
    let a = vec![Value::Null,
             Value::NullArray,
             Value::String("OKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOKOK"
                               .to_string()),
             Value::Error("ErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErrErr"
                              .to_string()),
             Value::Integer(1234567890),
             Value::Bulk("Bulk String Bulk String Bulk String Bulk String Bulk String Bulk String"
                             .to_string()),
             Value::Array(vec![Value::Null,
                               Value::Integer(123),
                               Value::Bulk("Bulk String Bulk String".to_string())])];
    let mut b = a.clone();
    b.push(Value::Array(a));
    b.push(Value::Null);

    let mut a = b.clone();
    a.push(Value::Array(b));
    a.push(Value::Null);

    Value::Array(a)
}

// Last result:
// test decode_values ... bench:       6,298 ns/iter (+/- 5,783)
// test encode_values ... bench:       2,568 ns/iter (+/- 1,879)

#[bench]
fn encode_values(b: &mut Bencher) {
    let value = prepare_values();
    b.iter(|| value.encode());
}

#[bench]
fn decode_values(b: &mut Bencher) {
    let value = prepare_values();
    let buf = value.encode();
    b.iter(|| {
               let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
               assert_eq!(decoder.decode().unwrap(), value);
               assert!(decoder.decode().is_err());
           });
}
