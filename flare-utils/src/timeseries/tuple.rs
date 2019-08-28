use std::fs::File;
use std::io::{SeekFrom, ErrorKind, Seek, Write, Read};
use chrono::Local;
use std::io::Error;
use std::io;
use byteorder::{WriteBytesExt, ReadBytesExt};
use std::str::from_utf8;
use num::FromPrimitive;
use std::cmp::min;

use super::TSEndian;


