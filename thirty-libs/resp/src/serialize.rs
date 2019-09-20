//! RESP serialize

use std::vec::Vec;
use std::string::String;
use std::io::{Read, BufRead, BufReader, Result, Error, ErrorKind};

use super::Value;

/// up to 512 MB in length
const RESP_MAX_SIZE: i64 = 512 * 1024 * 1024;
const CRLF_BYTES: &'static [u8] = b"\r\n";
const NULL_BYTES: &'static [u8] = b"$-1\r\n";
const NULL_ARRAY_BYTES: &'static [u8] = b"*-1\r\n";

/// Encodes RESP value to RESP binary buffer.
/// # Examples
/// ```
/// # use self::resp::{Value, encode};
/// let val = Value::String("OK".to_string());
/// assert_eq!(encode(&val), vec![43, 79, 75, 13, 10]);
/// ```
pub fn encode(value: &Value) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::with_capacity(1024);
    buf_encode(value, &mut res);
    res
}

/// Encodes a slice of string to RESP binary buffer.
/// It is use to create a request command on redis client.
/// # Examples
/// ```
/// # use self::resp::encode_slice;
/// let array = ["SET", "a", "1"];
/// assert_eq!(encode_slice(&array),
///            "*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n".to_string().into_bytes());
/// ```
pub fn encode_slice(slice: &[&str]) -> Vec<u8> {
    let array: Vec<Value> = slice.iter().map(|string| Value::Bulk(string.to_string())).collect();
    let mut res: Vec<u8> = Vec::new();
    buf_encode(&Value::Array(array), &mut res);
    res
}

#[inline]
fn buf_encode(value: &Value, buf: &mut Vec<u8>) {
    match *value {
        Value::Null => {
            buf.extend_from_slice(NULL_BYTES);
        }
        Value::NullArray => {
            buf.extend_from_slice(NULL_ARRAY_BYTES);
        }
        Value::String(ref val) => {
            buf.push(b'+');
            buf.extend_from_slice(val.as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
        }
        Value::Error(ref val) => {
            buf.push(b'-');
            buf.extend_from_slice(val.as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
        }
        Value::Integer(ref val) => {
            buf.push(b':');
            write_int(buf, val);
            //buf.extend_from_slice(val.to_string().as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
        }
        Value::Bulk(ref val) => {
            buf.push(b'$');
            write_int(buf, &(val.len() as i64));
            //buf.extend_from_slice(val.len().to_string().as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
            buf.extend_from_slice(val.as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
        }
        Value::BufBulk(ref val) => {
            buf.push(b'$');
            write_int(buf, &(val.len() as i64));
            //buf.extend_from_slice(val.len().to_string().as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
            buf.extend_from_slice(val);
            buf.extend_from_slice(CRLF_BYTES);
        }
        Value::Array(ref val) => {
            buf.push(b'*');
            write_int(buf, &(val.len() as i64));
            //buf.extend_from_slice(val.len().to_string().as_bytes());
            buf.extend_from_slice(CRLF_BYTES);
            for item in val {
                buf_encode(item, buf);
            }
        }
    }
}

//itoa is faster than int.to_string()
fn write_int(buf: &mut Vec<u8>, val: &i64) {
    //itoa::write(buf, *val);
    let mut itoa_buf = itoa::Buffer::new();
    let bytes = itoa_buf.format(*val).as_bytes();
    buf.extend_from_slice(bytes);
}

/// A streaming RESP Decoder.
#[derive(Debug)]
pub struct Decoder<R> {
    buf_bulk: bool,
    reader: BufReader<R>,
}

impl<R: Read> Decoder<R> {
    /// Creates a Decoder instance with given BufReader for decoding the RESP buffers.
    /// # Examples
    /// ```
    /// # use std::io::BufReader;
    /// # use self::resp::{Decoder, Value};
    ///
    /// let value = Value::Bulk("Hello".to_string());
    /// let buf = value.encode();
    /// let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
    /// assert_eq!(decoder.decode().unwrap(), Value::Bulk("Hello".to_string()));
    /// ```
    pub fn new(reader: BufReader<R>) -> Self {
        Decoder {
            buf_bulk: false,
            reader: reader,
        }
    }

    /// Creates a Decoder instance with given BufReader for decoding the RESP buffers.
    /// The instance will decode bulk value to buffer bulk.
    /// # Examples
    /// ```
    /// # use std::io::BufReader;
    /// # use self::resp::{Decoder, Value};
    ///
    /// let value = Value::Bulk("Hello".to_string());
    /// let buf = value.encode();
    /// let mut decoder = Decoder::with_buf_bulk(BufReader::new(buf.as_slice()));
    /// // Always decode "$" buffers to Value::BufBulk even if feed Value::Bulk buffers
    /// assert_eq!(decoder.decode().unwrap(), Value::BufBulk("Hello".to_string().into_bytes()));
    /// ```
    pub fn with_buf_bulk(reader: BufReader<R>) -> Self {
        Decoder {
            buf_bulk: true,
            reader: reader,
        }
    }

    /// It will read buffers from the inner BufReader, decode it to a Value.
    pub fn decode(&mut self) -> Result<Value> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        self.reader.read_until(b'\n', &mut res)?;

        let len = res.len();
        if len < 3 {
            return Err(Error::new(ErrorKind::InvalidInput, format!("too short: {}", len)));
        }
        if !is_crlf(res[len - 2], res[len - 1]) {
            return Err(Error::new(ErrorKind::InvalidInput, format!("invalid CRLF: {:?}", res)));
        }

        let bytes = res[1..len - 2].as_ref();
        match res[0] {
            // Value::String
            b'+' => parse_string(bytes).map(Value::String),
            // Value::Error
            b'-' => parse_string(bytes).map(Value::Error),
            // Value::Integer
            b':' => parse_integer(bytes).map(Value::Integer),
            // Value::Bulk
            b'$' => {
                let int = parse_integer(bytes)?;
                if int == -1 {
                    // Null bulk
                    return Ok(Value::Null);
                }
                if int < -1 || int >= RESP_MAX_SIZE {
                    return Err(Error::new(ErrorKind::InvalidInput,
                                          format!("invalid bulk length: {}", int)));
                }

                let int = int as usize;
//                let mut buf: Vec<u8> = Vec::new();
//                buf.resize(int+2, 0);
                // set_len is faster than vec.resize()
                let mut buf: Vec<u8> = Vec::with_capacity(int+2);
                unsafe { buf.set_len(int+2); }

                self.reader.read_exact(buf.as_mut_slice())?;
                if !is_crlf(buf[int], buf[int + 1]) {
                    return Err(Error::new(ErrorKind::InvalidInput,
                                          format!("invalid CRLF: {:?}", buf)));
                }
//                buf.truncate(int);
                unsafe { buf.set_len(int);}
                if self.buf_bulk {
                    return Ok(Value::BufBulk(buf));
                }
                parse_string(buf.as_slice()).map(Value::Bulk)
            }
            // Value::Array
            b'*' => {
                let int = parse_integer(bytes)?;
                if int == -1 {
                    // Null array
                    return Ok(Value::NullArray);
                }
                if int < -1 || int >= RESP_MAX_SIZE {
                    return Err(Error::new(ErrorKind::InvalidInput,
                                          format!("invalid array length: {}", int)));
                }

                let mut array: Vec<Value> = Vec::with_capacity(int as usize);
                for _ in 0..int {
                    let val = self.decode()?;
                    array.push(val);
                }
                Ok(Value::Array(array))
            }
            prefix => {
                Err(Error::new(ErrorKind::InvalidInput,
                               format!("invalid RESP type: {:?}", prefix)))
            }
        }
    }
}

#[inline]
fn is_crlf(a: u8, b: u8) -> bool {
    a == b'\r' && b == b'\n'
}

#[inline]
fn parse_string(bytes: &[u8]) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|err| Error::new(ErrorKind::InvalidData, err))
}

#[inline]
fn parse_integer(bytes: &[u8]) -> Result<i64> {
    let str_integer = parse_string(bytes)?;
    (str_integer.parse::<i64>()).map_err(|err| Error::new(ErrorKind::InvalidData, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Value;

    struct Case {
        data: Vec<u8>,
        want: Value,
    }

    #[test]
    fn fn_encode_slice() {
        let array = ["SET", "a", "1"];
        assert_eq!(String::from_utf8(encode_slice(&array)).unwrap(),
                   "*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n");

        let array = vec!["SET", "a", "1"];
        assert_eq!(String::from_utf8(encode_slice(&array)).unwrap(),
                   "*3\r\n$3\r\nSET\r\n$1\r\na\r\n$1\r\n1\r\n");
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
                  data: "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n"
                      .to_string()
                      .into_bytes(),
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
                  data: encode_slice(&vec!["SET", "a", "1"]),
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
    }

    #[test]
    fn struct_decoder_with_buf_bulk() {
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
                  want: Value::BufBulk("".to_string().into_bytes()),
              },
              Case {
                  data: "$6\r\nfoobar\r\n".to_string().into_bytes(),
                  want: Value::BufBulk("foobar".to_string().into_bytes()),
              },
              Case {
                  data: "$6\r\n中文\r\n".to_string().into_bytes(),
                  want: Value::BufBulk("中文".to_string().into_bytes()),
              },
              Case {
                  data: "$17\r\n你好！\n 换行\r\n".to_string().into_bytes(),
                  want: Value::BufBulk("你好！\n 换行".to_string().into_bytes()),
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
                  want: Value::Array(vec![Value::BufBulk("foo".to_string().into_bytes()),
                                          Value::BufBulk("bar".to_string().into_bytes())]),
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
                                          Value::BufBulk("foobar".to_string().into_bytes())]),
              },
              Case {
                  data: "*2\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n"
                      .to_string()
                      .into_bytes(),
                  want: Value::Array(vec![Value::Array(vec![Value::Integer(1),
                                                            Value::Integer(2),
                                                            Value::Integer(3)]),
                                          Value::Array(vec![Value::String("Foo".to_string()),
                                                            Value::Error("Bar".to_string())])]),
              },
              Case {
                  data: "*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nbar\r\n".to_string().into_bytes(),
                  want: Value::Array(vec![Value::BufBulk("foo".to_string().into_bytes()),
                                          Value::Null,
                                          Value::BufBulk("bar".to_string().into_bytes())]),
              },
              Case {
                  data: encode_slice(&vec!["SET", "a", "1"]),
                  want: Value::Array(vec![Value::BufBulk("SET".to_string().into_bytes()),
                                          Value::BufBulk("a".to_string().into_bytes()),
                                          Value::BufBulk("1".to_string().into_bytes())]),
              }];

        for case in cases {
            let mut decoder = Decoder::with_buf_bulk(BufReader::new(case.data.as_slice()));
            assert_eq!(decoder.decode().unwrap(), case.want);
            assert!(decoder.decode().is_err());
        }

        // Multiple Decode
        let mut all: Vec<u8> = Vec::new();
        for case in cases {
            all.extend_from_slice(case.data.as_slice());
        }
        let mut decoder = Decoder::with_buf_bulk(BufReader::new(all.as_slice()));
        for case in cases {
            assert_eq!(decoder.decode().unwrap(), case.want);
        }
        assert!(decoder.decode().is_err());
    }

    #[test]
    fn struct_decoder_with_invalid_data() {
        let buf: &[u8] = &[];
        let mut decoder = Decoder::new(BufReader::new(buf));
        assert!(decoder.decode().is_err());


        let buf = Value::String("OK正".to_string()).encode();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert_eq!(decoder.decode().unwrap(),
                   Value::String("OK正".to_string()));
        assert!(decoder.decode().is_err());

        let mut buf = Value::String("OK正".to_string()).encode();
        // [43, 79, 75, 230, 173, 163, 13, 10]
        buf.remove(5);
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());


        let buf = "$\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());

        let buf = "$-2\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());

        let buf = "&-1\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());

        let buf = "$-1\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert_eq!(decoder.decode().unwrap(), Value::Null);
        assert!(decoder.decode().is_err());

        let buf = "$0\r\n\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert_eq!(decoder.decode().unwrap(), Value::Bulk("".to_string()));
        assert!(decoder.decode().is_err());

        let buf = "*3\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());

        let buf = "*3\r\n$3\r\nfoo\r\n$-1\r\n".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());

        let buf = "*3\r\n$3\r\nfoo\r\n$-1\r\n$3\r\nba".to_string().into_bytes();
        let mut decoder = Decoder::new(BufReader::new(buf.as_slice()));
        assert!(decoder.decode().is_err());
    }
}
