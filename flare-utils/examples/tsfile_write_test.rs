
extern crate chrono;
//extern crate flareutils;

use flareutils::timeseries::*;
use chrono::Local;

fn main() {
    let mut tsfile = TimeSeriesFileWriter::new(ValueType::INT16, 100, "tsfile-test1.fts").unwrap();

    let now_time = Local::now().timestamp_millis();
    tsfile.add_value(now_time, TSValue::int16(1));
    tsfile.add_value(now_time+100, TSValue::int16(2));
    tsfile.add_value(now_time+150, TSValue::int16(3));

    let info = tsfile.get_header_info();
    println!("tsfile header: {:?}", info);

}