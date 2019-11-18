use std::fs::{File, OpenOptions};
use std::io::{SeekFrom, ErrorKind, Seek, Write, Read, BufReader};
use chrono::Local;
use std::io::Error;
use std::io;
use byteorder::{WriteBytesExt, ReadBytesExt, NetworkEndian};
use std::str::from_utf8;
use num::FromPrimitive;
use std::cmp::*;

use super::FileEndian;
use super::{ValueType, get_unit_len};
use crate::file_utils::open_file;
use std::collections::VecDeque;

#[derive(Clone, PartialEq, Debug)]
pub enum TSValue {
    int16(i16),
    int32(i32),
    int64(i64)
}

#[derive(Clone, PartialEq, Debug)]
pub enum TSRangeValue {
    vec_int16(Vec<i16>),
    vec_int32(Vec<i32>),
    vec_int64(Vec<i64>),
    vec_f32(Vec<f32>),
}

impl TSRangeValue {
    pub fn as_int64(&self) -> Option<Vec<i64>> {
        match self {
            TSRangeValue::vec_int64(x) => Some(x.clone()),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TSResult {
    pub begin_time: i64,
    pub end_time: i64,
    pub unit_time: i32,
    pub steps: i32,
    pub total_cpu_time: i64,
    pub data: TSRangeValue
}

#[derive( Debug )]
pub struct TimeSeriesFile {
    //file path
    pub path: String,
    //data segment start offset
    pub data_offset: u64,

    // value data type
    pub value_type: ValueType,
    // value unit len(bytes)
    pub unit_len: i8,
    //sample unit time ms
    pub unit_time: i32,
    // data start time ms
    pub begin_time: i64,
    // current time ms
    pub end_time: i64,
    // sample count
    pub amount: i32,
}

pub struct TimeSeriesFileReader {
    info: TimeSeriesFile,
    inited: bool,

}

pub struct TimeSeriesFileWriter {
    info: TimeSeriesFile,
    inited: bool,
    last_save_time: i64,
    last_sample_time: i64,

    //bulk value buffer
    data_buffer: VecDeque<(i64, TSValue)>,
    //last write bulk time
    last_flush_data_time: i64,
    //write interval
    data_flush_interval_time: i64,
    //cache value size limit
    data_buffer_size_limit: usize,

}

pub trait TimeSeries {

    fn get_header_info(&self) -> &TimeSeriesFile;

    fn get_begin_time(&self) -> i64;

    fn add_value(&mut self, time: i64, value: TSValue) -> Result<u32, Error>;

    fn get_range_value(&self, start_time: i64, end_time: i64, unit_time_ms: i32) -> TSResult;

    fn time_to_step(&self, time: i64) -> u32 {
        let info = self.get_header_info();
        let mut steps = (time - info.begin_time) / info.unit_time as i64;
        if steps < 0 {
            steps = 0;
        }
        steps as u32
    }
}

impl TimeSeriesFile {

    fn new(value_type: ValueType, unit_time: i32, path: &str, file: File) -> TimeSeriesFile {
        TimeSeriesFile{
            path: path.to_string(),
            data_offset: 0,
            unit_time,
            unit_len: get_unit_len(value_type),
            value_type,
            begin_time: 0,
            end_time: 0,
            amount: 0,
        }
    }

    fn get_file(&self) -> Result<File, Error> {
        Ok(open_file(&self.path, true)?)
    }

    pub fn get_range_value(&self, origin_start_time: i64, origin_end_time: i64, unit_time_ms: i32) -> TSResult {
        let mut data_vec = vec![];
        // self.begin_time <= start_time <= self.end_time
        let mut start_time = max(self.begin_time, origin_start_time);
        start_time = min(start_time, self.end_time);

        // self.begin_time <= end_time <= self.end_time
        let mut end_time = max(self.begin_time, origin_end_time);
        end_time = min(end_time, self.end_time);

        //convert time to steps
        let step1 = (start_time - self.begin_time) / self.unit_time as i64;
        let step2 = (end_time - self.begin_time) / self.unit_time as i64;

        //convert steps to file offset
        let offset1 = self.data_offset + self.unit_len as u64 * step1 as u64;
        let offset2 = self.data_offset + self.unit_len as u64 * step2 as u64;

        //read specify range data
        match self.get_file() {
            Ok(mut file) => {
                file.seek(SeekFrom::Start(offset1));
                //let mut stream = file.take(offset2 - offset1);
                //stream.read_i16_into::<FileEndian>(data_vec.as_mut_slice());
                let bytes = offset2 - offset1;
                let mut buf_reader = BufReader::with_capacity(1024*100, file);
                let steps = bytes / self.unit_len as u64;

                match self.value_type {
                    ValueType::UNKNOWN => {},
                    ValueType::INT16 => {
                        for i in 0..steps {
                            data_vec.push(buf_reader.read_i16::<FileEndian>().unwrap() as i64);
                        }
                    },
                    ValueType::UINT16 => {
                        for i in 0..steps {
                            data_vec.push(buf_reader.read_u16::<FileEndian>().unwrap() as i64);
                        }
                    },
                    ValueType::INT32 => {
                        for i in 0..steps {
                            data_vec.push(buf_reader.read_i32::<FileEndian>().unwrap() as i64);
                        }
                    },
                    ValueType::UINT32 => {
                        for i in 0..steps {
                            data_vec.push(buf_reader.read_u32::<FileEndian>().unwrap() as i64);
                        }
                    },
                    ValueType::INT64 => {
                        for i in 0..steps {
                            data_vec.push(buf_reader.read_i64::<FileEndian>().unwrap() as i64);
                        }
                    },
//            ValueType::FLOAT64 => {},
                }
            },
            Err(e) => {
                println!("open ts file failed, path: {}, error: {}", self.path, e);
            }
        }


        //convert time unit, merge n source point to one new point
        let merge_num = (unit_time_ms / self.unit_time) as usize;
        let mut unit_time_ms = merge_num as i32 * self.unit_time;
        if merge_num > 1 {
            let mut new_data_vec = vec![];
            let size = data_vec.len() / merge_num as usize;
            //fill data
            for i in 0..size {
                new_data_vec.push(ts_sum_int64(&data_vec[i*merge_num..(i+1)*merge_num]));
            }
            let total_cpu_time = ts_sum_int64(new_data_vec.as_slice());
            TSResult {
                begin_time: start_time,
                end_time,
                total_cpu_time,
                unit_time: unit_time_ms,
                steps: new_data_vec.len() as i32,
                data: TSRangeValue::vec_int64(new_data_vec)
            }
        }else {
            let total_cpu_time = ts_sum_int64(data_vec.as_slice());
            TSResult {
                begin_time: start_time,
                end_time,
                total_cpu_time,
                unit_time: unit_time_ms,
                steps: data_vec.len() as i32,
                data: TSRangeValue::vec_int64(data_vec)
            }
        }
    }
}

//填充空的数据，使得返回的时序数据范围的一致的
fn fill_null_data(mut data_vec: Vec<i64>, start_time: i64, end_time: i64, origin_start_time: i64, origin_end_time: i64, unit_time_ms: i32) -> Vec<i64> {
    let fill_steps_before = (start_time - origin_start_time)/unit_time_ms as i64;
    let fill_steps_after = (origin_end_time - end_time)/unit_time_ms as i64;
    if fill_steps_before == 0 && fill_steps_after == 0 {
        return data_vec;
    }

    let mut new_data_vec = Vec::with_capacity(data_vec.len()+(fill_steps_before+fill_steps_after) as usize);
    for i in 0..fill_steps_before {
        new_data_vec.push(0);
    }

    new_data_vec.append(&mut data_vec);

    for i in 0..fill_steps_after {
        new_data_vec.push(0);
    }
    new_data_vec
}

fn ts_avg_int16 (numbers: &[i16]) -> f32 {
    let mut sum = 0;
    numbers.iter().for_each(|x| sum += *x as i32);
    sum as f32 / numbers.len() as f32
}

fn ts_sum_int16 (numbers: &[i16]) -> i64 {
    let mut sum = 0;
    numbers.iter().for_each(|x| sum += *x as i64);
    sum
}

fn ts_sum_int64 (numbers: &[i64]) -> i64 {
    let mut sum = 0;
    numbers.iter().for_each(|x| sum += *x as i64);
    sum
}

//fn average(numbers: &[i32]) -> f32 {
//    numbers.iter().sum::<i32>() as f32 / numbers.len() as f32
//}
//
//fn median(numbers: &mut [i32]) -> i32 {
//    numbers.sort();
//    let mid = numbers.len() / 2;
//    numbers[mid]
//}


impl TimeSeriesFileReader {

    pub fn new(path: &str) -> Result<TimeSeriesFileReader, Error> {
        let mut path = path.to_string()+".fts";
        //let now_time = Local::now().timestamp_millis();
        match File::open(path.clone()) {
            Ok(file) => {
                let info = TimeSeriesFile::new(ValueType::UNKNOWN, 0, &path, file);
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
            let file = &mut info.get_file()?;
            file.seek(SeekFrom::Start(0));
            //TS file header segment: TSHS (4 bytes)
            file.read_exact(&mut flag_buf[..]);
            let flag = std::str::from_utf8(&flag_buf[..]).unwrap();
            if flag != "TSHS" {
                println!("Invalid time series file, header segment flag not match");
                return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid time series file (header segment)"));
            }

            //header len (2 bytes)
            let header_len = file.read_u16::<FileEndian>().unwrap() as u64;
            let header_offset = 4 + 2;

            //header data (n bytes)
            info.value_type = ValueType::from_i8(file.read_i8().unwrap()).unwrap();
            info.unit_len = file.read_i8().unwrap();
            info.unit_time = file.read_i32::<FileEndian>().unwrap();
            info.begin_time = file.read_i64::<FileEndian>().unwrap();
            info.end_time = file.read_i64::<FileEndian>().unwrap();
            info.amount = file.read_i32::<FileEndian>().unwrap();

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

}

impl TimeSeries for TimeSeriesFileReader {

    fn get_header_info(&self) -> &TimeSeriesFile {
        &self.info
    }

    fn get_begin_time(&self) -> i64 {
        self.info.begin_time
    }

    fn add_value(&mut self, time: i64, value: TSValue) -> Result<u32, Error> {
        Err(Error::new(ErrorKind::PermissionDenied, "can not modify data file in reader mode"))
    }

    fn get_range_value(&self, start_time: i64, end_time: i64, unit_time_ms: i32) -> TSResult {
        self.info.get_range_value(start_time, end_time, unit_time_ms)
    }
}


impl TimeSeriesFileWriter {

    pub fn new(value_type: ValueType, unit_time: i32, begin_time: i64, path: &str) -> Result<TimeSeriesFileWriter, Error> {
        let data_flush_interval_time = 1000;
        let data_buffer_size_limit = 1000;

        let mut path = path.to_string()+".fts";
        let now_time = Local::now().timestamp_millis();
        let file_rs = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone());
        match file_rs {
            Ok(file) => {
                let info = TimeSeriesFile::new(value_type, unit_time, &path, file);
                let mut writer = TimeSeriesFileWriter {
                    info,
                    inited: false,
                    last_save_time: 0,
                    last_sample_time: 0,
                    data_buffer: VecDeque::with_capacity(data_buffer_size_limit+2),
                    data_buffer_size_limit,
                    last_flush_data_time: 0,
                    data_flush_interval_time,
                };
                writer.init(begin_time);
                Ok(writer)
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
        header_vec.write_i32::<FileEndian>(info.unit_time);
        header_vec.write_i64::<FileEndian>(info.begin_time);
        header_vec.write_i64::<FileEndian>(info.end_time);
        header_vec.write_i32::<FileEndian>(info.amount);

        //write file header
        match info.get_file() {
            Ok(mut file) => {
                file.seek(SeekFrom::Start(0));

                //TS file header segment: TSHS (4 bytes)
                file.write_all(b"TSHS");

                //header len (2 bytes)
                file.write_u16::<FileEndian>(header_vec.len() as u16);

                //header data (n bytes)
                file.write_all(header_vec.as_slice());

                //data segment flag: TSDS
                file.write_all(b"TSDS");

                //save data segment start offset
                info.data_offset = file.seek(SeekFrom::Current(0)).unwrap();
                //info.data_offset = file.stream_position().unwrap();

                self.last_save_time = self.last_sample_time;
            },
            Err(e) =>{
                println!("save ts file header_info failed, path: {}, error: {}", self.info.path, e);
            }
        }

    }

    pub fn flush(&mut self) -> Result<(), Error> {
        let info = &mut self.info;
        let file = &mut info.get_file()?;

        while let Some((steps, value)) = self.data_buffer.pop_front() {
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
                    file.write_i16::<FileEndian>(val);
                }
                TSValue::int32(val) => {
                    if info.value_type != ValueType::INT32 {
                        println!("value type not match, expect {:?} but {:?}", info.value_type, ValueType::INT32);
                        return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
                    }
                    let offset = (steps * 4) as u64;
                    file.seek(SeekFrom::Start(info.data_offset + offset));
                    file.write_i32::<FileEndian>(val);
                }
                TSValue::int64(val) => {
                    if info.value_type != ValueType::INT64 {
                        println!("value type not match, expect {:?} but {:?}", info.value_type, ValueType::INT64);
                        return Err(io::Error::new(ErrorKind::InvalidInput, "value type not match"));
                    }
                    let offset = (steps * 8) as u64;
                    file.seek(SeekFrom::Start(info.data_offset + offset));
                    file.write_i64::<FileEndian>(val);
                }
                _ => {
                    println!("unsupported value type: {:?}", info.value_type);
                    return Err(io::Error::new(ErrorKind::InvalidInput, "unsupported value type"));
                }
            }
        }

        //save header info periodically
//        self.last_sample_time = time;
//        if time - self.last_save_time > 1000 {
//        }
        self.save_header_info();
        let now_time = Local::now().timestamp_millis();
        self.last_flush_data_time = now_time;

        Ok(())
    }

}

impl TimeSeries for TimeSeriesFileWriter {

    fn get_header_info(&self) -> &TimeSeriesFile {
        &self.info
    }

    fn get_begin_time(&self) -> i64 {
        self.info.begin_time
    }

    fn add_value(&mut self, time: i64, value: TSValue) -> Result<u32, Error> {

        let info = &mut self.info;
        let mut steps = (time - info.begin_time) / info.unit_time as i64;
        if steps < 0 {
            steps = 0;
        }
        self.data_buffer.push_back((steps, value));

        //flush
        //TODO call get timestamp_millis() may be frequently, maybe too heavily?
        let now_time = Local::now().timestamp_millis();
        let interval = now_time - self.last_flush_data_time;
        if interval > self.data_flush_interval_time || self.data_buffer.len() >= self.data_buffer_size_limit {
            println!("flushing ts file, data_buffer len:{}, write interval: {} ...", self.data_buffer.len(), interval);
            self.flush();
        }

        Ok(steps as u32)
    }

    fn get_range_value(&self, start_time: i64, end_time: i64, unit_time_ms: i32) -> TSResult {
        self.info.get_range_value(start_time, end_time, unit_time_ms)
    }
}

impl Drop for TimeSeriesFileWriter {

    fn drop(&mut self) {
        self.flush();
        self.save_header_info();
    }

}