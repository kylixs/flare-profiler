# RESP

RESP(REdis Serialization Protocol) Serialization for Rust.

[![Crates version][version-image]][version-url]
[![Build Status][travis-image]][travis-url]
[![Coverage Status][coveralls-image]][coveralls-url]
[![Crates downloads][downloads-image]][downloads-url]
[![Docs Status][docs-image]][docs-url]

Implementations:

- [redis-cli](https://github.com/iorust/redis-cli) redis CLI.

## API

```Rust
extern crate resp;
use resp::{Value, encode, encode_slice, Decoder};
```

### RESP Values

```Rust
enum Value {
    /// Null bulk reply, $-1\r\n
    Null,
    /// Null array reply, *-1\r\n
    NullArray,
    /// For Simple Strings the first byte of the reply is "+"
    String(String),
    /// For Errors the first byte of the reply is "-"
    Error(String),
    /// For Integers the first byte of the reply is ":"
    Integer(i64),
    /// For Bulk Strings the first byte of the reply is "$"
    Bulk(String),
    /// For Bulk <binary> Strings the first byte of the reply is "$"
    BufBulk(Vec<u8>),
    /// For Arrays the first byte of the reply is "*"
    Array(Vec<Value>),
}
```

#### `value.is_null() -> bool`

#### `value.is_error() -> bool`

#### `value.encode() -> Vec<u8>`

#### `value.to_encoded_string() -> io::Result<String>`

#### `value.to_beautify_string() -> String`

### encode

#### `fn encode(value: &Value) -> Vec<u8>`

#### `fn encode_slice(array: &[&str]) -> Vec<u8>`

### Decoder

#### `Decoder.new(reader: BufReader<R>) -> Self`

#### `Decoder.with_buf_bulk(reader: BufReader<R>) -> Self`

#### `decoder.decode() -> Result<Value>`


[version-image]: https://img.shields.io/crates/v/resp.svg
[version-url]: https://crates.io/crates/resp

[travis-image]: http://img.shields.io/travis/iorust/resp.svg
[travis-url]: https://travis-ci.org/iorust/resp

[coveralls-image]: https://coveralls.io/repos/github/iorust/resp/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/iorust/resp?branch=master

[downloads-image]: https://img.shields.io/crates/d/resp.svg
[downloads-url]: https://crates.io/crates/resp

[docs-image]: https://docs.rs/resp/badge.svg
[docs-url]: https://docs.rs/resp
