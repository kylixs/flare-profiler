#![warn(missing_docs)]
#![doc(html_logo_url = "https://avatars3.githubusercontent.com/u/15439811?v=3&s=200",
       html_favicon_url = "https://iorust.github.io/favicon.ico",
       html_root_url = "https://iorust.github.io",
       html_playground_url = "https://play.rust-lang.org",
       issue_tracker_base_url = "https://github.com/iorust/resp/issues")]

//! RESP(Redis Serialization Protocol) Serialization for Rust.

pub use self::value::Value;
pub use self::serialize::{encode, encode_slice, Decoder};

mod value;
mod serialize;
