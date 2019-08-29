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

