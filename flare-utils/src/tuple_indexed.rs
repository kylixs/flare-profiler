// indexed file

use std::fs::{File, OpenOptions};
use std::io::{SeekFrom, ErrorKind, Seek, Write, Read, BufReader};
use chrono::Local;
use std::io::Error;
use std::io;
use byteorder::{WriteBytesExt, ReadBytesExt, NetworkEndian};
use std::str::from_utf8;
use num::{FromPrimitive, PrimInt};
use std::collections::HashMap;
use std::cmp::*;

use super::FileEndian;
use super::file_utils::*;
use super::collections::*;
use crate::collections::MapUtil::*;

//bulk data handler
type BulkDataConsumer = fn(Vec<u8>);

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
        UINT16,
        INT32,
        UINT32,
        INT64,
        FLOAT64
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TupleValue {
    int16(i16),
    uint16(u16),
    int32(i32),
    uint32(u32),
    int64(i64),
//    float64(f64)
}

fn get_value_type(value: &TupleValue) -> ValueType {
    match value {
        TupleValue::int16(_) => ValueType::INT16,
        TupleValue::uint16(_) => ValueType::UINT16,
        TupleValue::int32(_) => ValueType::INT32,
        TupleValue::uint32(_) => ValueType::UINT32,
        TupleValue::int64(_) => ValueType::INT64,
//        TupleValue::float64(_) => ValueType::FLOAT64,
        _ => ValueType::UNKNOWN
    }
}

///
/// save tuple values in indexed file
#[derive( Debug )]
pub struct TupleIndexedFile {
    //state
    inited: bool,
    writable: bool,
    index_map: HashMap<TupleValue, TupleValue>,

    //indexed file
    indexed_file: File,
    indexed_path: String,
    //data segment start offset
    indexed_data_offset: u64,

    //extra data file
    extra_file: File,
    extra_path: String,
    extra_data_offset: u64,

    //header info
    // index value type
    pub index_type: ValueType,
    // bulk data type
    pub bulk_offset_type: ValueType,
    // tuple unit len(bytes)
    pub unit_len: i8,
    // data start time ms
    pub begin_time: i64,
    // current time ms
    pub end_time: i64,
    // sample count
    pub amount: i32,
}

//init methods
impl TupleIndexedFile {

    pub fn new_reader(path: &str) -> Result<TupleIndexedFile, io::Error> {
        let mut tuple_file = TupleIndexedFile::new(path,
                                                   ValueType::UNKNOWN,
                                                   ValueType::UNKNOWN,
                                                   false)?;
        tuple_file.init_reader()?;
        Ok(tuple_file)
    }

    pub fn new_writer(path: &str, index_type: ValueType) -> Result<TupleIndexedFile, io::Error> {

        let mut tuple_file = TupleIndexedFile::new(path,
                                                   index_type,
                                                   ValueType::UINT32,
                                                   true)?;
        tuple_file.init_writer()?;
        Ok(tuple_file)
    }

    fn new(path: &str, index_type: ValueType, bulk_offset_type: ValueType, writable: bool) -> Result<TupleIndexedFile, io::Error> {
        let indexed_path = path.to_string() + ".fidx";
        let extra_path = path.to_string() + ".fdata";
        let unit_len = get_unit_len(index_type) + get_unit_len(bulk_offset_type);
        let indexed_file = open_file(&indexed_path, writable)?;
        let extra_file = open_file(&extra_path, writable)?;
        Ok(TupleIndexedFile {
            inited: false,
            writable,
            index_map: HashMap::new(),
            indexed_file,
            indexed_path,
            indexed_data_offset: 0,
            extra_file,
            extra_path,
            extra_data_offset: 0,
            index_type,
            bulk_offset_type,
            unit_len,
            begin_time: 0,
            end_time: 0,
            amount: 0
        })
    }

    fn init_reader(&mut self) -> Result<(), Error> {
        if !self.inited {
            self.load_indexed_header_info()?;
            self.load_extra_header_info()?;
            self.load_index_map()?;
            self.inited = true;
        }
        Ok(())
    }

    fn init_writer(&mut self) -> Result<(), Error> {
        if !self.inited {
            let now_time = Local::now().timestamp_millis();
            self.begin_time = now_time;
            self.end_time = now_time;

            self.save_indexed_header_info()?;
            self.save_extra_header_info()?;
            self.inited = true;
        }
        Ok(())
    }

    fn save_indexed_header_info(&mut self) -> Result<(), Error> {

        let mut header_map = HashMap::new();
        header_map.insert("first_el_type", (self.index_type as i8).to_string());
        header_map.insert("second_el_type", (self.bulk_offset_type as i8).to_string());
        header_map.insert("unit_len", self.unit_len.to_string());
        header_map.insert("begin_time", self.begin_time.to_string());
        header_map.insert("end_time", self.end_time.to_string());
        header_map.insert("amount", self.amount.to_string());

        //write file header
        let file = &mut self.indexed_file;
        file.seek(SeekFrom::Start(0));

        write_header_info(file, &header_map, TUPLE_INDEXED_HEADER_SEGMENT_FLAG, TUPLE_INDEXED_DATA_SEGMENT_FLAG)?;

        //save data segment start offset
        self.indexed_data_offset = file.seek(SeekFrom::Current(0)).unwrap();
        //info.data_offset = file.stream_position().unwrap();
        Ok(())
    }

    fn save_extra_header_info(&mut self) -> Result<(), Error> {

        let mut header_map = HashMap::new();
        header_map.insert("first_el_type", (self.index_type as i8).to_string());
        header_map.insert("second_el_type", (self.bulk_offset_type as i8).to_string());
        header_map.insert("unit_len", self.unit_len.to_string());
        header_map.insert("begin_time", self.begin_time.to_string());
        header_map.insert("end_time", self.end_time.to_string());
        header_map.insert("amount", self.amount.to_string());

        //write file header
        let file = &mut self.extra_file;
        file.seek(SeekFrom::Start(0));

        write_header_info(file, &header_map, TUPLE_EXTRA_HEADER_SEGMENT_FLAG, TUPLE_EXTRA_DATA_SEGMENT_FLAG)?;

        //save data segment start offset
        self.extra_data_offset = file.seek(SeekFrom::Current(0)).unwrap();
        //info.data_offset = file.stream_position().unwrap();
        Ok(())
    }

    fn load_indexed_header_info(&mut self) -> Result<(), io::Error> {
        //read indexed file header
        let file = &mut self.indexed_file;
        file.seek(SeekFrom::Start(0));

        let mut header_map: HashMap<String, String> = HashMap::new();
        self.indexed_data_offset = read_header_info(file, &mut header_map, TUPLE_INDEXED_HEADER_SEGMENT_FLAG, TUPLE_INDEXED_DATA_SEGMENT_FLAG)?;

        self.index_type = ValueType::from_i8(get_as_i8(&mut header_map, "first_el_type")).unwrap();
        self.bulk_offset_type = ValueType::from_i8(get_as_i8(&mut header_map, "second_el_type")).unwrap();
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

//read & write value methods
impl TupleIndexedFile {

    pub fn add_value(&mut self, index: TupleValue, bulk_value: &[u8]) -> io::Result<()> {
        let val_type = get_value_type(&index);
        if val_type != self.index_type {
            println!("index type not match, expect '{:?}' but '{:?}'", self.index_type, val_type );
            return Err(io::Error::new(ErrorKind::InvalidInput, "index type not match"));
        }

        let bulk_offset = self.extra_file.seek(SeekFrom::End(0)).unwrap();
        //bulk len
        self.extra_file.write_u16::<FileEndian>(bulk_value.len() as u16)?;
        //bulk data
        self.extra_file.write_all(bulk_value)?;

        //index value
        self.write_indexed_value(&index, self.index_type);
        //bulk offset
        let bulk_offset_value = TupleValue::uint32(bulk_offset as u32);
        self.write_indexed_value(&bulk_offset_value, self.bulk_offset_type);

        self.index_map.insert(index, bulk_offset_value);
        self.amount += 1;

        Ok(())
    }

    fn write_indexed_value(&mut self, value: &TupleValue, expect_value_type: ValueType) -> io::Result<()> {
        let val_type = get_value_type(&value);
        if val_type != expect_value_type {
            println!("value type not match, expect '{:?}' but '{:?}'", expect_value_type, val_type );
            return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
        }

        let file = &mut self.indexed_file;
        file.seek(SeekFrom::End(0));
        match value {
            TupleValue::int16(v) => {
                file.write_i16::<FileEndian>(*v);
            },
            TupleValue::uint16(v) => {
                file.write_u16::<FileEndian>(*v);
            },
            TupleValue::int32(v) => {
                file.write_i32::<FileEndian>(*v);
            },
            TupleValue::uint32(v) => {
                file.write_u32::<FileEndian>(*v);
            },
            TupleValue::int64(v) => {
                file.write_i64::<FileEndian>(*v);
            },
//            TupleValue::float64(v) => {
//                file.write_f64::<FileEndian>(*v);
//            },
//            _ => {
//                println!("unsupported indexed value: {:?}", value);
//                return Err(io::Error::new(ErrorKind::InvalidInput, "unsupported indexed value"));
//            }
        }
        Ok(())
    }

    fn read_indexed_value(reader: &mut Read, index_type: &ValueType) -> io::Result<TupleValue> {
        match index_type {
            ValueType::INT16 => {
                Ok(TupleValue::int16(reader.read_i16::<FileEndian>()?))
            },
            ValueType::UINT16 => {
                Ok(TupleValue::uint16(reader.read_u16::<FileEndian>()?))
            },
            ValueType::INT32 => {
                Ok(TupleValue::int32(reader.read_i32::<FileEndian>()?))
            },
            ValueType::UINT32 => {
                Ok(TupleValue::uint32(reader.read_u32::<FileEndian>()?))
            },
            ValueType::INT64 => {
                Ok(TupleValue::int64(reader.read_i64::<FileEndian>()?))
            },
//            ValueType::FLOAT64 => {
//
//            },
            _ => {
                println!("unsupported read index value: {:?}", index_type);
                Err(io::Error::new(ErrorKind::InvalidInput, "unsupported read index value"))
            },
        }
    }

    fn load_index_map(&mut self) -> io::Result<()> {
        let file = &mut self.indexed_file;
        file.seek(SeekFrom::Start(self.indexed_data_offset));

        let mut reader = BufReader::new(file);
        loop {
            if let Ok(index_value) = TupleIndexedFile::read_indexed_value(&mut reader, &self.index_type) {
                if let Ok(bulk_offset_value) = TupleIndexedFile::read_indexed_value(&mut reader, &self.bulk_offset_type) {
                    self.index_map.insert(index_value, bulk_offset_value);
                } else {
                    break;
                }
            }else {
                break;
            }
        }
        Ok(())
    }

    ///
    /// read bulk value by index
    ///
    pub fn get_value(&mut self, index: &TupleValue) -> io::Result<Vec<u8>> {
        let mut bulk_offset = 0u32;
        if let Some(TupleValue::uint32(offset)) = self.index_map.get(index) {
            bulk_offset = *offset;
        } else {
            return Err(io::Error::new(ErrorKind::NotFound, "index not found"));
        }
        let (buf, offset) = self.read_bulk_data(bulk_offset)?;
        Ok(buf)

    }

    fn read_bulk_data(&mut self, bulk_offset: u32) -> Result<(Vec<u8>,u32), Error> {
        self.extra_file.seek(SeekFrom::Start(bulk_offset as u64));
        let bytes_to_read = self.extra_file.read_u16::<FileEndian>()? as usize;
        let mut buf = vec![0u8; bytes_to_read];
        self.extra_file.read_exact(&mut buf)?;
        let new_offset = self.extra_file.seek(SeekFrom::Current(0)).unwrap();
        Ok((buf, new_offset as u32))
    }

    pub fn get_range_value(&mut self, start_index: &TupleValue, end_index: &TupleValue, handler: BulkDataConsumer) -> io::Result<()> {
        let mut start_offset = 0u32;
        let mut end_offset = 0u32;
        let mut found = false;
        if let Some(TupleValue::uint32(offset1)) = self.index_map.get(start_index) {
            start_offset = *offset1;
            if let Some(TupleValue::uint32(offset2)) = self.index_map.get(end_index) {
                end_offset = *offset2;
                found = true;
            }
        }
        if found {
            let mut offset = start_offset;
            loop {
                let (buf, next_offset) = self.read_bulk_data(offset)?;
                handler(buf);
                offset = next_offset;
                if offset > end_offset {
                    break;
                }
            }
            Ok(())
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "index not found"))
        }
    }
}

impl Drop for TupleIndexedFile {

    fn drop(&mut self) {
        if self.writable {
            self.save_indexed_header_info();
        }
    }

}