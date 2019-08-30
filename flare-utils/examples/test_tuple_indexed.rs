

extern crate rand;
extern crate chrono;
//extern crate flareutils;

use flareutils::tuple_indexed::*;
use chrono::Local;
use std::{io, fs};

fn main() -> io::Result<()> {
//    let aa = "aaaa\0";
//    let bb = aa.trim().to_string();
//    let cc = aa.trim_matches(|x|x=='\0' ).to_string();
//    println!("trim: {} - {} - {}", aa.trim(), bb, cc);
    let path = "tuple-test1";
    fs::remove_file(path.to_string()+".fidx");
    fs::remove_file(path.to_string()+".fdata");

    let mut tuple_file = TupleIndexedFile::new_writer(path, ValueType::INT32)?;
    tuple_file.add_value(get_key(100), b"first line");
    tuple_file.add_value(get_key(120), &[0,1,1,2]);
    tuple_file.add_value(get_key(130), &[23,1,1,2,5,6]);
    tuple_file.add_value(get_key(150), &[120,1,21,24,2,7,8,2]);

    println!("get value: {:?}", tuple_file.get_value(&get_key(100)));
    println!("get value: {:?}", tuple_file.get_value(&get_key(150)));

    drop(tuple_file);

    let mut tuple_file = TupleIndexedFile::new_reader(path)?;
    println!("tuple file: {:?}", tuple_file);
    println!("get value: {:?}", tuple_file.get_value(&get_key(100)));
    println!("get value: {:?}", tuple_file.get_value(&get_key(150)));

    let start_index = get_key(100);
    let end_index = get_key(150);
    println!("test get_range_value: {:?} - {:?}", start_index, end_index);
    tuple_file.get_range_value(&start_index, &end_index, |vec| {
        println!("value: {:?}", vec);
    });

    Ok(())
}

fn get_key(index: i32) -> TupleValue {
    TupleValue::int32(index)
}