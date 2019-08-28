//re-export
pub use self::unary::*;
pub use self::tuple::*;

mod unary;
mod tuple;


use byteorder::{LittleEndian, WriteBytesExt, NetworkEndian, ReadBytesExt};

//TS file byte order
type TSEndian = NetworkEndian;