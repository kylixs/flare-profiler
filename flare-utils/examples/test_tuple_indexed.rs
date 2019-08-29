

extern crate rand;
extern crate chrono;
//extern crate flareutils;

use flareutils::tuple_indexed::*;
use chrono::Local;
use std::io;

fn main() -> io::Result<()> {
//    let aa = "aaaa\0";
//    let bb = aa.trim().to_string();
//    let cc = aa.trim_matches(|x|x=='\0' ).to_string();
//    println!("trim: {} - {} - {}", aa.trim(), bb, cc);

    let mut tuple_file = TupleIndexedFile::new_writer("tuple-test1", ValueType::INT32)?;
    tuple_file.add_value(get_key(100), b"first line");
    tuple_file.add_value(get_key(120), &[0,1,1,2]);
    tuple_file.add_value(get_key(130), &[23,1,1,2,5,6]);
    tuple_file.add_value(get_key(150), &[120,1,21,24,2,7,8,2]);

    println!("get value: {:?}", tuple_file.get_value(&get_key(100)));
    println!("get value: {:?}", tuple_file.get_value(&get_key(150)));

    drop(tuple_file);

    let mut tuple_file = TupleIndexedFile::new_reader("tuple-test1")?;
    println!("tuple file: {:?}", tuple_file);
    println!("get value: {:?}", tuple_file.get_value(&get_key(100)));
    println!("get value: {:?}", tuple_file.get_value(&get_key(150)));

    Ok(())
}

fn get_key(index: i32) -> TupleValue {
    TupleValue::int32(index)
}