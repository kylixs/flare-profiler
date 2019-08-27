use std::fs::File;
use std::io::{SeekFrom, ErrorKind, Seek, Write, BufWriter, Read};
use chrono::Local;
use std::io::Error;
use std::io;
use byteorder::{LittleEndian, WriteBytesExt, NetworkEndian, ReadBytesExt};
use std::str::from_utf8;
use std::fmt::{Display, Formatter};

use num::FromPrimitive;
use std::cmp::min;

//TS file byte order
type TSEndian = NetworkEndian;

#[derive(Clone, PartialEq)]
pub enum TSValue {
    int16(i16),
    int32(i32),
    int64(i64)
}

enum_from_primitive! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum ValueType {
        UNKNOWN,
        INT16,
        INT32,
        INT64
    }
}

pub fn get_unit_len(value_type: ValueType) -> i8{
    match value_type {
        INT16 => 2,
        INT32 => 4,
        INT64 => 8
    }
}

#[derive( Debug )]
pub struct TimeSeriesFile {
    //file path
    path: String,
    //file object
    file: File,
    //data segment start offset
    data_offset: u64,

    // value data type
    value_type: ValueType,
    // value unit len(bytes)
    unit_len: i8,
    //sample unit time ms
    unit_time: i32,
    // data start time ms
    begin_time: i64,
    // current time ms
    end_time: i64,
    // sample count
    amount: i32,
}

pub struct TimeSeriesFileReader {
    info: TimeSeriesFile,
    inited: bool,

}

pub struct TimeSeriesFileWriter {
    info: TimeSeriesFile,
    inited: bool,

}

impl TimeSeriesFile {

    fn new(value_type: ValueType, unit_time: i32, path: &str, file: File) -> TimeSeriesFile {
        TimeSeriesFile{
            path: path.to_string(),
            file: file,
            data_offset: 0,
            unit_time: unit_time,
            unit_len: get_unit_len(value_type),
            value_type: value_type,
            begin_time: 0,
            end_time: 0,
            amount: 0
        }
    }
}

//impl Display for TimeSeriesFile {
//
//    fn fmt(&self, f: &mut Formatter) -> Result<(), core::fmt::Error> {
//        Display::fmt("{  }", f)
//    }
//}

impl TimeSeriesFileReader {

    pub fn new(path: &str) -> Result<TimeSeriesFileReader, Error> {
        //let now_time = Local::now().timestamp_millis();
        match File::open(path.clone()) {
            Ok(file) => {
                let info = TimeSeriesFile::new(ValueType::UNKNOWN, 0, path, file);
                let mut reader = TimeSeriesFileReader {
                    info: info,
                    inited: false
                };
                reader.init();
                Ok(reader)
            },
            Err(err) => Err(err)
        }
    }

    fn init(&mut self) -> Result<bool, Error> {
        if !self.inited {
            let info = &mut self.info;

            //read file header
            let mut flag_buf = [0 as u8;4];
            let file = &mut info.file;
            file.seek(SeekFrom::Start(0));
            //TS file header segment: TSHS (4 bytes)
            file.read_exact(&mut flag_buf[..]);
            let flag = std::str::from_utf8(&flag_buf[..]).unwrap();
            if flag != "TSHS" {
                println!("Invalid time series file, header segment flag not match");
                return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid time series file (header segment)"));
            }

            //header len (2 bytes)
            let header_len = file.read_u16::<TSEndian>().unwrap() as u64;
            let header_offset = 4 + 2;

            //header data (n bytes)
            info.value_type = ValueType::from_i8(file.read_i8().unwrap()).unwrap();
            info.unit_len = file.read_i8().unwrap();
            info.unit_time = file.read_i32::<TSEndian>().unwrap();
            info.begin_time = file.read_i64::<TSEndian>().unwrap();
            info.end_time = file.read_i64::<TSEndian>().unwrap();
            info.amount = file.read_i32::<TSEndian>().unwrap();

            //data segment flag
            file.seek(SeekFrom::Start(header_offset + header_len));
            file.read_exact(&mut flag_buf[..]);
            let flag = std::str::from_utf8(&flag_buf[..]).unwrap();
            if flag != "TSDS" {
                println!("Invalid time series file, data segment flag not match: {}", flag);
                return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid time series file (data segment)"));
            }

            //save data segment start offset
            info.data_offset = header_offset + header_len + 4;

            self.inited = true;
        }
        Ok(true)
    }

    pub fn get_header_info(&mut self) -> &TimeSeriesFile {
        self.init();
        &self.info
    }

    pub fn get_value_in_time_range(&mut self) {
        self.init();

    }
}


impl TimeSeriesFileWriter {

    pub fn new(value_type: ValueType, unit_time: i32, path: &str) -> Result<TimeSeriesFileWriter, Error> {
        let now_time = Local::now().timestamp_millis();
        match File::create(path.clone()) {
            Ok(file) => {
                let info = TimeSeriesFile::new(value_type, unit_time, path, file);
                Ok(TimeSeriesFileWriter {
                    info: info,
                    inited: false
                })
            },
            Err(err) => Err(err)
        }
    }

    fn init(&mut self, time: i64) -> Result<bool, Error> {

        if !self.inited {
            let info = &mut self.info;
            info.begin_time = time;
            info.end_time = time;

            self.save_header_info();

            self.inited = true;
        }
        Ok(true)
    }

    fn save_header_info(&mut self) {
        let info = &mut self.info;
        //encode header
        let mut header_vec = vec![];
        header_vec.write_i8(info.value_type as i8);
        header_vec.write_i8(info.unit_len);
        header_vec.write_i32::<TSEndian>(info.unit_time);
        header_vec.write_i64::<TSEndian>(info.begin_time);
        header_vec.write_i64::<TSEndian>(info.end_time);
        header_vec.write_i32::<TSEndian>(info.amount);

        //write file header
        let file = &mut info.file;
        file.seek(SeekFrom::Start(0));

        //TS file header segment: TSHS (4 bytes)
        file.write_all(b"TSHS");

        //header len (2 bytes)
        file.write_u16::<TSEndian>(header_vec.len() as u16);

        //header data (n bytes)
        file.write_all(header_vec.as_slice());

        //data segment flag: TSDS
        file.write_all(b"TSDS");

        //save data segment start offset
        info.data_offset = file.seek(SeekFrom::Current(0)).unwrap();
        //info.data_offset = file.stream_position().unwrap();

    }

    pub fn get_header_info(&mut self) -> &TimeSeriesFile {
        &self.info
    }

    pub fn add_value(&mut self, time: i64, value: TSValue) -> Result<bool, Error> {
        self.init(time);
        let info = &mut self.info;
        let steps = (time - info.begin_time) / info.unit_time as i64;
        let file = &mut info.file;
        info.amount = min(info.amount+1, steps as i32 +1);
        info.end_time = info.begin_time + steps * info.unit_time as i64;

        match value {
            TSValue::int16(val) => {
                if info.value_type != ValueType::INT16 {
                    println!("value type not match, expect {:?} but {:?}", info.value_type, ValueType::INT16);
                    return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
                }
                let offset = (steps * 2) as u64;
                file.seek(SeekFrom::Start(info.data_offset + offset));
                file.write_i16::<TSEndian>(val);
            }
            TSValue::int32(val) => {
                if info.value_type != ValueType::INT32 {
                    println!("value type not match, expect {:?} but {:?}", info.value_type, ValueType::INT32);
                    return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
                }
                let offset = (steps * 4) as u64;
                file.seek(SeekFrom::Start(info.data_offset + offset));
                file.write_i32::<TSEndian>(val);
            }
            TSValue::int64(val) => {
                if info.value_type != ValueType::INT64 {
                    println!("value type not match, expect {:?} but {:?}", info.value_type, ValueType::INT64);
                    return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
                }
                let offset = (steps * 8) as u64;
                file.seek(SeekFrom::Start(info.data_offset + offset));
                file.write_i64::<TSEndian>(val);
            }
            _ => {
                println!("unsupported value type: {:?}", info.value_type);
                return Err(io::Error::new(ErrorKind::InvalidInput, "unsupported value type"));
            }
        }

        Ok(true)
    }
}

impl Drop for TimeSeriesFileWriter {

    fn drop(&mut self) {
        self.save_header_info();
    }

}