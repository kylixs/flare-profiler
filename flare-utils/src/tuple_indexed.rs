// indexed file

use std::fs::{File, OpenOptions};
use std::io::{SeekFrom, ErrorKind, Seek, Write, Read, BufReader};
use chrono::Local;
use std::io::Error;
use std::io;
use byteorder::{WriteBytesExt, ReadBytesExt, NetworkEndian};
use std::str::from_utf8;
use num::{FromPrimitive, PrimInt};
use std::cmp::*;

use super::FileEndian;
use super::file_utils::*;
use std::collections::HashMap;

//Tuple-Indexed file Header Segment: TIHS (4 bytes)
static TUPLE_INDEXED_HEADER_SEGMENT_FLAG: &str = "TIHS";
//Tuple-Indexed Data Segment flag: TIDS
static TUPLE_INDEXED_DATA_SEGMENT_FLAG: &str = "TIDS";

//Tuple-Extra file Header Segment: TEHS (4 bytes)
static TUPLE_EXTRA_HEADER_SEGMENT_FLAG: &str = "TEHS";
//Tuple-Extra Data Segment flag: TEDS
static TUPLE_EXTRA_DATA_SEGMENT_FLAG: &str = "TEDS";

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum ValueType {
        UNKNOWN,
        INT16,
        INT32,
        INT64,
        FLOAT64
    }
}

pub fn get_unit_len(value_type: ValueType) -> i8{
    match value_type {
        INT16 => 2,
        INT32 => 4,
        INT64 => 8,
        FLOAT64=> 8
    }
}

///
/// save tuple values in indexed file
#[derive( Debug )]
pub struct TupleIndexedFile {
    //state
    inited: bool,
    writable: bool,

    //indexed file
    indexed_file: File,
    pub indexed_path: String,
    //data segment start offset
    pub indexed_data_offset: u64,

    //extra data file
    extra_file: File,
    pub extra_path: String,
    pub extra_data_offset: u64,

    //header info
    // first el data type
    pub first_el_type: ValueType,
    // first el data type
    pub second_el_type: ValueType,
    // tuple unit len(bytes)
    pub unit_len: i8,
    // data start time ms
    pub begin_time: i64,
    // current time ms
    pub end_time: i64,
    // sample count
    pub amount: i32,
}

impl TupleIndexedFile {

    pub fn new_reader(path: &str) -> Result<TupleIndexedFile, io::Error> {
        let mut tuple_file = TupleIndexedFile::new(path, ValueType::UNKNOWN, ValueType::UNKNOWN, false)?;
        tuple_file.init_reader()?;
        Ok(tuple_file)
    }

    pub fn new_writer(path: &str, first_el_type: ValueType, second_el_type: ValueType) -> Result<TupleIndexedFile, io::Error> {
        let mut tuple_file = TupleIndexedFile::new(path, first_el_type, second_el_type, true)?;
        tuple_file.init_writer()?;
        Ok(tuple_file)
    }

    fn new(path: &str, first_el_type: ValueType, second_el_type: ValueType, writable: bool) -> Result<TupleIndexedFile, io::Error> {
        let indexed_path = path.to_string() + ".tpidx";
        let extra_path = path.to_string() + ".tpdata";
        let unit_len = get_unit_len(first_el_type) + get_unit_len(second_el_type);
        let indexed_file = open_file(&indexed_path, writable)?;
        let extra_file = open_file(&extra_path, writable)?;
        Ok(TupleIndexedFile {
            inited: false,
            writable,
            indexed_file,
            indexed_path,
            indexed_data_offset: 0,
            extra_file,
            extra_path,
            extra_data_offset: 0,
            first_el_type,
            second_el_type,
            unit_len,
            begin_time: 0,
            end_time: 0,
            amount: 0
        })
    }

    fn init_reader(&mut self) -> Result<bool, Error> {
        if !self.inited {
            self.load_indexed_header_info()?;
            self.load_extra_header_info()?;
            self.inited = true;
        }
        Ok(true)
    }

    fn init_writer(&mut self) -> Result<bool, Error> {
        if !self.inited {
            let now_time = Local::now().timestamp_millis();
            self.begin_time = now_time;
            self.end_time = now_time;

            self.save_indexed_header_info();
            self.save_extra_header_info();
            self.inited = true;
        }
        Ok(true)
    }

    fn save_indexed_header_info(&mut self) {

        let mut header_map = HashMap::new();
        header_map.insert("first_el_type", (self.first_el_type as i8).to_string());
        header_map.insert("second_el_type", (self.second_el_type as i8).to_string());
        header_map.insert("unit_len", self.unit_len.to_string());
        header_map.insert("begin_time", self.begin_time.to_string());
        header_map.insert("end_time", self.end_time.to_string());
        header_map.insert("amount", self.amount.to_string());

        //write file header
        let file = &mut self.indexed_file;
        file.seek(SeekFrom::Start(0));

        write_header_info(file, &header_map, TUPLE_INDEXED_HEADER_SEGMENT_FLAG, TUPLE_INDEXED_DATA_SEGMENT_FLAG);

        //save data segment start offset
        self.indexed_data_offset = file.seek(SeekFrom::Current(0)).unwrap();
        //info.data_offset = file.stream_position().unwrap();
    }

    fn save_extra_header_info(&mut self) {

        let mut header_map = HashMap::new();
        header_map.insert("first_el_type", (self.first_el_type as i8).to_string());
        header_map.insert("second_el_type", (self.second_el_type as i8).to_string());
        header_map.insert("unit_len", self.unit_len.to_string());
        header_map.insert("begin_time", self.begin_time.to_string());
        header_map.insert("end_time", self.end_time.to_string());
        header_map.insert("amount", self.amount.to_string());

        //write file header
        let file = &mut self.extra_file;
        file.seek(SeekFrom::Start(0));

        write_header_info(file, &header_map, TUPLE_EXTRA_HEADER_SEGMENT_FLAG, TUPLE_EXTRA_DATA_SEGMENT_FLAG);

        //save data segment start offset
        self.extra_data_offset = file.seek(SeekFrom::Current(0)).unwrap();
        //info.data_offset = file.stream_position().unwrap();
    }

    fn load_indexed_header_info(&mut self) -> Result<(), io::Error> {
        //read indexed file header
        let file = &mut self.indexed_file;
        file.seek(SeekFrom::Start(0));

        let mut header_map: HashMap<String, String> = HashMap::new();
        self.indexed_data_offset = read_header_info(file, &mut header_map, TUPLE_INDEXED_HEADER_SEGMENT_FLAG, TUPLE_INDEXED_DATA_SEGMENT_FLAG)?;

        self.first_el_type = ValueType::from_i8(get_as_i8(&mut header_map, "first_el_type")).unwrap();
        self.second_el_type = ValueType::from_i8(get_as_i8(&mut header_map, "second_el_type")).unwrap();
        self.begin_time = get_as_i64(&mut header_map, "begin_time");
        self.end_time = get_as_i64(&mut header_map, "end_time");
        self.unit_len = get_as_i8(&mut header_map, "unit_len");
        self.amount = get_as_i32(&mut header_map, "amount");
        Ok(())
    }

    fn load_extra_header_info(&mut self) -> Result<(), io::Error> {
        //read extra file header

        let file = &mut self.extra_file;
        file.seek(SeekFrom::Start(0));

        let mut header_map: HashMap<String, String> = HashMap::new();
        self.extra_data_offset = read_header_info(file, &mut header_map, TUPLE_EXTRA_HEADER_SEGMENT_FLAG, TUPLE_EXTRA_DATA_SEGMENT_FLAG)?;
        Ok(())
    }
}

fn get_as_i64(header_map: &mut HashMap<String, String>, key: &str) -> i64 {
    header_map.get(key).unwrap().parse::<i64>().unwrap()
}

fn get_as_i32(header_map: &mut HashMap<String, String>, key: &str) -> i32 {
    header_map.get(key).unwrap().parse::<i32>().unwrap()
}

fn get_as_i8(header_map: &mut HashMap<String, String>, key: &str) -> i8 {
    header_map.get(key).unwrap().parse::<i8>().unwrap()
}

impl Drop for TupleIndexedFile {

    fn drop(&mut self) {
        if self.writable {
            self.save_indexed_header_info();
        }
    }

}