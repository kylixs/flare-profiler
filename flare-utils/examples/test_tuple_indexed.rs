

extern crate rand;
extern crate chrono;
//extern crate flareutils;

use flareutils::tuple_indexed::*;
use chrono::Local;

fn main() {
    let aa = "aaaa\0";
    let bb = aa.trim().to_string();
    let cc = aa.trim_matches(|x|x=='\0' ).to_string();
    println!("trim: {} - {} - {}", aa.trim(), bb, cc);

    let tuple_file = TupleIndexedFile::new_writer("tuple-test1", ValueType::INT32, ValueType::INT64);
    drop(tuple_file);

    let tuple_file = TupleIndexedFile::new_reader("tuple-test1");
    println!("tuple file: {:?}", tuple_file);
}