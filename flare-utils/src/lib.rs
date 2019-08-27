#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate chrono;
#[macro_use]
extern crate enum_primitive;
extern crate num;

pub mod timeseries;
