extern crate flare_utils;

use std::collections::HashMap;
use flare_utils::stopwatch::Stopwatch;

fn main() {

    test_entry_and_insert_with();

    test_get_and_insert();

}

fn test_entry_and_insert_with() {
    let mut map :HashMap<i64, i64> = HashMap::new();
    let mut stopwatch = Stopwatch::new();
    stopwatch.start();
    for i in 0..500000 {
        let value = map.entry(i).or_insert_with(||{
            i
        });
    }
    let cost = stopwatch.elapsed_ms();
    println!("test_entry_and_insert_with cost: {}ms", cost);
}

fn test_get_and_insert() {
    let mut map :HashMap<i64, i64> = HashMap::new();
    let mut stopwatch = Stopwatch::new();
    stopwatch.start();
    for i in 0..500000 {
        let mut value = map.get_mut(&i);
        if value.is_none() {
            map.insert(i, i);
        }
        value = map.get_mut(&i);
    }
    let cost = stopwatch.elapsed_ms();
    println!("test_get_and_insert cost: {}ms", cost);
}