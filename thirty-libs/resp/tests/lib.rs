//! RESP and serialization

extern crate resp;
extern crate rand;

use std::io;
use std::io::{Read, BufReader};
use resp::{Value, encode, encode_slice, Decoder};
use std::{thread, time};
use rand::{StdRng, Rng};

struct Case {
    data: Vec<u8>,
    want: Value,
}

struct FakeNetIO {
    offset: usize,
    buf: Vec<u8>,
}

impl Read for FakeNetIO {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset < self.buf.len() {
            let mut rng = StdRng::new().unwrap();
            let mut n: usize = rng.gen_range(0, std::cmp::min(buf.len(), 1024));
            if n == 0 {
                n = buf.len();
            }
            if self.offset + n > self.buf.len() {
                n = self.buf.len() - self.offset;
            }
            thread::sleep(time::Duration::new(0, n as u32));
            buf[..n].copy_from_slice(&self.buf[self.offset..self.offset + n]);
            self.offset += n;
            return Ok(n);
        }
        // Ok(0)
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"))
    }
}

#[test]
fn enum_is_null() {
    assert_eq!(Value::Null.is_null(), true);
    assert_eq!(Value::NullArray.is_null(), true);
    assert_eq!(Value::Integer(123).is_null(), false);
}

#[test]
fn enum_is_error() {
    assert_eq!(Value::Null.is_error(), false);
    assert_eq!(Value::NullArray.is_error(), false);
    assert_eq!(Value::Error("".to_string()).is_error(), true);
}

#[test]
fn enum_encode() {
    let val = Value::String("OK正".to_string());
    assert_eq!(val.encode(), vec![43, 79, 75, 230, 173, 163, 13, 10]);
}

#[test]
fn enum_to_encoded_string() {
    let val = Value::String("OK正".to_string());
    assert_eq!(val.to_encoded_string().unwrap(), "+OK正\r\n");
}

#[test]
fn enum_to_beautify_string() {
    assert_eq!(Value::Null.to_beautify_string(), "(Null)");
    assert_eq!(Value::NullArray.to_beautify_string(), "(Null Array)");
    assert_eq!(Value::String("OK".to_string()).to_beautify_string(), "OK");
    assert_eq!(Value::Error("Err".to_string()).to_beautify_string(),
               "(Error) Err");
    assert_eq!(Value::Integer(123).to_beautify_string(), "(Integer) 123");
    assert_eq!(Value::Bulk("Bulk String".to_string()).to_beautify_string(),
               "\"Bulk String\"");
    assert_eq!(Value::BufBulk(vec![]).to_beautify_string(),
               "(Empty Buffer)");
    assert_eq!(Value::BufBulk(vec![0, 100]).to_beautify_string(),
               "(Buffer) 00 64");
    assert_eq!(Value::BufBulk(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
                                   18])
                       .to_beautify_string(),
               "(Buffer) 00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f ...");
    assert_eq!(Value::Array(vec![]).to_beautify_string(), "(Empty Array)");
    assert_eq!(Value::Array(vec![Value::Null, Value::Integer(123)]).to_beautify_string(),
               "1) (Null)\n2) (Integer) 123");
}

#[test]
fn fn_encode() {
    let val = Value::String("OK正".to_string());
    assert_eq!(encode(&val), vec![43, 79, 75, 230, 173, 163, 13, 10]);
}

#[test]
fn fn_encode_slice() {
    let array = ["SET", "a", "1"];
    assert_eq!(encode_slice(&array),
               "*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n".to_string().into_bytes());
}

#[test]
fn struct_decoder() {
    let cases: &[Case] =
        &[Case {
              data: "+\r\n".to_string().into_bytes(),
              want: Value::String("".to_string()),
          },
          Case {
              data: "+OK\r\n".to_string().into_bytes(),
              want: Value::String("OK".to_string()),
          },
          Case {
              data: "+中文\r\n".to_string().into_bytes(),
              want: Value::String("中文".to_string()),
          },
          Case {
              data: "-Error message\r\n".to_string().into_bytes(),
              want: Value::Error("Error message".to_string()),
          },
          Case {
              data: ":-1\r\n".to_string().into_bytes(),
              want: Value::Integer(-1),
          },
          Case {
              data: ":0\r\n".to_string().into_bytes(),
              want: Value::Integer(0),
          },
          Case {
              data: ":1456061893587000000\r\n".to_string().into_bytes(),
              want: Value::Integer(1456061893587000000),
          },
          Case {
              data: "$-1\r\n".to_string().into_bytes(),
              want: Value::Null,
          },
          Case {
              data: "$0\r\n\r\n".to_string().into_bytes(),
              want: Value::Bulk("".to_string()),
          },
          Case {
              data: "$6\r\nfoobar\r\n".to_string().into_bytes(),
              want: Value::Bulk("foobar".to_string()),
          },
          Case {
              data: "$6\r\n中文\r\n".to_string().into_bytes(),
              want: Value::Bulk("中文".to_string()),
          },
          Case {
              data: "$17\r\n你好！\n 换行\r\n".to_string().into_bytes(),
              want: Value::Bulk("你好！\n 换行".to_string()),
          },
          Case {
              data: "*-1\r\n".to_string().into_bytes(),
              want: Value::NullArray,
          },
          Case {
              data: "*0\r\n".to_string().into_bytes(),
              want: Value::Array(vec![]),
          },
          Case {
              data: "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_string().into_bytes(),
              want: Value::Array(vec![Value::Bulk("foo".to_string()),
                                      Value::Bulk("bar".to_string())]),
          },
          Case {
              data: "*3\r\n:1\r\n:2\r\n:3\r\n".to_string().into_bytes(),
              want: Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]),
          },
          Case {
              data: "*5\r\n:1\r\n:2\r\n:3\r\n:4\r\n$6\r\nfoobar\r\n".to_string().into_bytes(),
              want: Value::Array(vec![Value::Integer(1),
                                      Value::Integer(2),
                                      Value::Integer(3),
                                      Value::Integer(4),
                                      Value::Bulk("foobar".to_string())]),
          },
          Case {
              data: "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n".to_string().into_bytes(),
              want: Value::Array(vec![Value::Array(vec![Value::Integer(1),
                                                        Value::Integer(2),
                                                        Value::Integer(3)]),
                                      Value::Array(vec![Value::String("Foo".to_string()),
                                                        Value::Error("Bar".to_string())])]),
          },
          Case {
              data: "*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nbar\r\n".to_string().into_bytes(),
              want: Value::Array(vec![Value::Bulk("foo".to_string()),
                                      Value::Null,
                                      Value::Bulk("bar".to_string())]),
          },
          Case {
              data: encode_slice(&["SET", "a", "1"]),
              want: Value::Array(vec![Value::Bulk("SET".to_string()),
                                      Value::Bulk("a".to_string()),
                                      Value::Bulk("1".to_string())]),
          }];

    // Single Decode
    for case in cases {
        let mut decoder = Decoder::new(BufReader::new(case.data.as_slice()));
        assert_eq!(decoder.decode().unwrap(), case.want);
        assert!(decoder.decode().is_err());
    }

    // Multiple Decode
    let mut all: Vec<u8> = Vec::new();
    for case in cases {
        all.extend_from_slice(case.data.as_slice());
    }
    let mut decoder = Decoder::new(BufReader::new(all.as_slice()));
    for case in cases {
        assert_eq!(decoder.decode().unwrap(), case.want);
    }
    assert!(decoder.decode().is_err());

    // Chaos Decode
    let repeats: usize = 100000;
    let mut chaos: Vec<u8> = Vec::new();
    for _ in 0..repeats {
        chaos.extend_from_slice(all.as_slice());
    }
    println!("8888888 {:?}", chaos.len());
    let mut decoder = Decoder::new(BufReader::new(FakeNetIO {
                                                      offset: 0,
                                                      buf: chaos,
                                                  }));
    for _ in 0..repeats {
        for case in cases {
            assert_eq!(decoder.decode().unwrap(), case.want);
        }
    }
    assert!(decoder.decode().is_err());
}
