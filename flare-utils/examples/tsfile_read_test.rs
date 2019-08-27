
extern crate chrono;

use flareutils::timeseries::*;
use chrono::Local;

fn main() {
    let mut tsfile = TimeSeriesFileReader::new("tsfile-test1.fts").unwrap();
    let info = tsfile.get_header_info();
    println!("tsfile header: {:?}", info);

}