
pub mod MapUtil {
    use std::collections::HashMap;

    pub fn get_as_i64(map: &mut HashMap<String, String>, key: &str) -> i64 {
        map.get(key).unwrap().parse::<i64>().unwrap()
    }

    pub fn get_as_i32(map: &mut HashMap<String, String>, key: &str) -> i32 {
        map.get(key).unwrap().parse::<i32>().unwrap()
    }

    pub fn get_as_i8(map: &mut HashMap<String, String>, key: &str) -> i8 {
        map.get(key).unwrap().parse::<i8>().unwrap()
    }
}


