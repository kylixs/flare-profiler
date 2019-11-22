extern crate chrono;
extern crate flare_utils;

use flare_utils::tuple_indexed::*;
use chrono::Local;
use std::{io, fs};
use flare_utils::file_utils::open_file;
use std::io::{Seek, SeekFrom, Write};

fn main() -> io::Result<()> {

    //TODO 注意： 此修复方法只针对旧的文件格式，不适用于新版

    let samples_dir = "D:\\projects\\arch\\flare-profiler\\flare-server\\flare-samples\\";
    let dirs =  std::fs::read_dir(samples_dir)?;
    for dir in dirs {
        //sample
        let path_buf = dir.unwrap().path();
        if !std::fs::metadata(&path_buf).unwrap().is_dir() {
            continue;
        }
        let paths =  std::fs::read_dir(path_buf)?;
        for path in paths {
            let path_str = path.unwrap().path().to_str().unwrap().to_owned();
            if path_str.ends_with(".fidx") {
                let idx_path = &path_str[0..path_str.len()-5];
                let mut tuple_file = TupleIndexedFile::new_reader(idx_path)?;
                if tuple_file.amount > 0 {
                    println!("fidx file: {}, amount: {}, data: {:?}", path_str, tuple_file.amount, tuple_file.get_index_pairs(0,3));
                    println!("try fix fidx file ...");
                    let mut idx_file = TupleIndexedFile::new_writer(idx_path, tuple_file.index_type)?;
                    //针对旧版索引文件头部损坏的修复
//                    idx_file.amount = 0;
//                    idx_file.save_indexed_header_info()?;
                    let indexed_data_offset = idx_file.indexed_data_offset;
                    //
                    //let mut indexed_file = open_file(&indexed_path, writable)?;
//                    let mut indexed_file = &idx_file.indexed_file;
//                    indexed_file.seek(SeekFrom::Start(indexed_data_offset));
//                    indexed_file.write_all(&[0;4]);

                    //after fix
                    let tuple_file = TupleIndexedFile::new_reader(idx_path)?;
                    println!("after fixed fidx file: {}, amount: {}, data: {:?}", path_str, tuple_file.amount, tuple_file.get_index_pairs(0,3));

                }
            }
        }
    }
    Ok(())
}