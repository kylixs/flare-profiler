
use std::fs::{OpenOptions, File};
use std::io;
use std::collections::HashMap;
//use eclectic::map::*;

use super::{FileEndian,WriteBytesExt,ReadBytesExt};
use std::io::{Write, Read, ErrorKind, BufReader, BufRead, Seek, SeekFrom};

//open file with read and write permissions
pub fn open_file(path: &str, rw: bool) -> Result<File, io::Error> {
    OpenOptions::new()
        .read(true)
        .write(rw)
        .create(rw)
        .open(path.to_string())
}

//write common file header
pub fn write_header_info(file: &mut File, header_map: &HashMap<&str, String>, header_segment_flag: &str, data_segment_flag: &str) -> Result<u64, io::Error> {
    //encode header
    let mut header_vec = vec![];
    //property size (1 byte)
    header_vec.write_i8(header_map.len() as i8);
    for (name, value) in header_map.iter() {
        header_vec.write_all(name.as_bytes());
        header_vec.write_u8(0);
        header_vec.write_all(value.as_bytes());
        header_vec.write_u8(0);
    }
    let max_len = 192;
    if header_vec.len() > max_len {
        return Err(io::Error::new(ErrorKind::InvalidInput, "header len is too large!"));
    }
    //添加一个空闲空间，避免后面重写头部覆盖数据
    let pad_len = max_len - header_vec.len();
    let mut pad_data = Vec::new();
    pad_data.resize(pad_len, 0);
    header_vec.write_all(&pad_data);

    //write file header
    //TS file header segment: TSHS (4 bytes)
    file.write_all(header_segment_flag.as_bytes())?;

    //header len (2 bytes)
    file.write_u16::<FileEndian>(header_vec.len() as u16)?;

    //header data (n bytes)
    file.write_all(header_vec.as_slice())?;

    //data segment flag: TSDS
    file.write_all(data_segment_flag.as_bytes())?;
    file.flush();

    Ok(file.seek(SeekFrom::Current(0)).unwrap())
}

pub fn read_header_info(file: &mut File, header_map: &mut HashMap<String, String>, header_segment_flag: &str, data_segment_flag: &str) -> Result<u64, io::Error> {
    //read file header
    let flag = read_file_flag(file);
    if flag != header_segment_flag {
        println!("Invalid file, header segment flag not match, expect '{}' but '{}'", header_segment_flag, flag);
        return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid file, header segment not match"));
    }

    //header len (2 bytes)
    let header_len = file.read_u16::<FileEndian>().unwrap() as u64;
    let header_offset = 4 + 2;
    let header_count = file.read_u8().unwrap();

    let mut buf_reader = BufReader::new(file as &mut Read);
    let reader = &mut buf_reader;
    for i in 00..header_count {
        let name = read_utf8(reader);
        let value = read_utf8(reader);
        header_map.insert(name.to_string(), value.to_string());
    }

    //verify data segment flag
    file.seek(SeekFrom::Start(header_offset+header_len));
    let flag = read_file_flag(file);
    if flag != data_segment_flag {
        println!("Invalid file, data segment flag not match, expect '{}' but '{}'", data_segment_flag, flag);
        return Err(io::Error::new(ErrorKind::InvalidInput, "Invalid file, data segment not match"));
    }
    Ok(file.seek(SeekFrom::Current(0)).unwrap())
}

fn read_utf8(buf_reader: &mut BufReader<&mut Read>) -> String {
    let mut buf = vec![];
    let num_bytes = buf_reader.read_until(b'\0', &mut buf)
        .expect("expect delimiter '\0'");
    let s = std::str::from_utf8(buf.as_slice()).unwrap();
    s.trim_matches(|x|x=='\0' ).to_string()
}

fn read_file_flag(file: &mut Read) -> String {
    let mut flag_buf = [0 as u8; 4];
//TS file header segment: TSHS (4 bytes)
    file.read_exact(&mut flag_buf[..]);
    let flag = std::str::from_utf8(&flag_buf[..]).unwrap();
    flag.to_string()
}

