extern crate resp;

use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use resp::{Value, Decoder};

fn main() {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 3333");

//            let msg = b"Hello!";
//
//            stream.write(msg).unwrap();
//            println!("Sent Hello, awaiting reply...");
            let cmdValue = Value::Array(vec![Value::String("start-sample".to_string()),
                                          Value::String("interval".to_string()),
                                          Value::Integer(20)]);

            let cmd = cmdValue.encode();

            let size = stream.write(cmd.as_slice()).unwrap();
            println!("Sent cmd, awaiting reply: {}", cmdValue.to_encoded_string().unwrap());


            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
//                    if &data == msg {
//                        println!("Reply is ok!");
//                    } else {
//                        let text = from_utf8(&data).unwrap();
//                        println!("Unexpected reply: {}", text);
//                    }

                    let text = from_utf8(&data).unwrap();
                    println!("reply: {}", text);
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}