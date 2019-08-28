
extern crate rand;
extern crate chrono;
//extern crate flareutils;

use flareutils::timeseries::*;
use chrono::Local;

fn main() {
    let unit_time = 100 as i64;
    let mut tsfile = TimeSeriesFileWriter::new(ValueType::INT16, unit_time as i32, "tsfile-test1.fts").unwrap();

    let start_time = Local::now().timestamp_millis();
    for i in 0..20000 {
        tsfile.add_value(start_time+i*unit_time, TSValue::int16(1000+ i as i16));
    }

    let info = tsfile.get_header_info();
    println!("tsfile header: {:?}", info);

    let t1 = Local::now().timestamp_millis();
    let tsresult = tsfile.get_range_value_int16(start_time + unit_time*20, start_time + unit_time*18000, 5 * unit_time as i32);
    let t2 = Local::now().timestamp_millis();

    println!("result: {:?}, cost: {}ms", tsresult, (t2-t1));
}