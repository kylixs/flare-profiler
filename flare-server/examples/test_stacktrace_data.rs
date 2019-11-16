
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate resp;
extern crate chrono;
extern crate flare_utils;
extern crate flare_server;

use flare_server::sample::ThreadData;
use flare_server::utils::*;
use flare_utils::stopwatch::Stopwatch;
use flare_server::sample_encoder::*;
use std::{io, mem};
use std::io::BufReader;

fn main() -> io::Result<()> {

    let mut thread_data = ThreadData {
        id: 73,
        name: "http-nio-8400-exec-5".into(),
        priority: 5,
        daemon: false,
        state: "".into(),
        cpu_time: 3560000000,
        cpu_time_delta: 1560000,
        sample_time: 1568812237809,
        stacktrace: vec![484014896,483776904,547216200,547216208,547254368,547254376,547254384,547254392,547254400,547254408,547254416,547254424,547252208,547252216,547241752,547241984,546904800,483779488,483779496,547241992,547240960,547240968,547237224,547237232,547237240,547237200,547235120,547235128,547236800,547236808,547236816,547236824,547233000,547233008,547236832,547233000,547233008,547235096,547232992,547233000,547233008,547235104,547232992,547233000,547233008,547235112,547232992,547233000,547233008,547234832,547233000,547233008,547234840,547232992,547233000,547233008,547230792,547230752,547230480,547230488,547228864,547228872,547227784,547227424,547227432,547224168,547224176,547224184,483776608,483776616,547195736,483776624]
    };

    let run_times = 10000;
    let json = serde_json::to_string(&thread_data)?;
    println!("json: {}", json);

    let mut sw = Stopwatch::start_new();
    let mut json_vec = Vec::with_capacity(run_times);
    for i in 0..run_times {
        thread_data.id += 1;
        json_vec.push(serde_json::to_string(&thread_data)?);
    }
    println!("to json: {}", sw.lap());

    let mut thread_data_vec = Vec::with_capacity(run_times);
    for s in json_vec {
        let obj = serde_json::from_str::<ThreadData>(&s)?;
        thread_data_vec.push(obj);
    }
    println!("parse json: {}", sw.lap());


    let mut resp_vec = Vec::with_capacity(run_times);
    let resp_value = resp_encode_thread_data(&thread_data).encode();
    for i in 0..run_times {
        thread_data.id += 1;
        resp_vec.push(resp_encode_thread_data(&thread_data).encode() );
    }
    println!("to resp: {}", sw.lap());


    let mut thread_data_vec = Vec::with_capacity(run_times);
//    let mut tmp_vec = Vec::with_capacity(run_times);
    for buf in resp_vec {
        let val = resp::Decoder::with_buf_bulk(BufReader::new(buf.as_slice())).decode()?;
        if let resp::Value::Array(data_vec) = &val {
            thread_data_vec.push(resp_decode_thread_data(data_vec));
        }
//        tmp_vec.push(val);
    }
    println!("parse resp: {}", sw.lap());
    println!("parse thread_data: {}", serde_json::to_string(&thread_data_vec[0])?);

    Ok(())
}

fn test_json() {

}

fn test_resp() {

}