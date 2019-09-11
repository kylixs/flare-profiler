#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate chrono;
#[macro_use]
extern crate enum_primitive;
extern crate num;
//extern crate eclectic;


pub mod timeseries;
pub mod tuple_indexed;
pub mod file_utils;
pub mod collections;

use byteorder::{WriteBytesExt, ReadBytesExt, NetworkEndian};
use std::io;

//default file byte order
type FileEndian = NetworkEndian;


enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum ValueType {
        UNKNOWN,
        INT16,
        UINT16,
        INT32,
        UINT32,
        INT64,
//        FLOAT64
    }
}

fn get_unit_len(value_type: ValueType) -> i8{
    match value_type {
        INT16 => 2,
        UINT16 => 2,
        INT32 => 4,
        UINT32 => 4,
        INT64 => 8,
//        FLOAT64=> 8
    }
}