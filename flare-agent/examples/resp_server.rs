extern crate resp;
#[macro_use]
extern crate lazy_static;

use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use resp::{Value, Decoder};
use std::io::BufReader;
use std::sync::{Mutex,Arc,RwLock};


lazy_static! {
    static ref RUNNING_SERVER: Mutex<bool> = Mutex::new(false);
}

fn set_server_running(val: bool) {
    let mut a = RUNNING_SERVER.lock().unwrap();
    *a = val;
}

fn is_server_running() -> bool {
    *RUNNING_SERVER.lock().unwrap()
}

fn close_server() {
    set_server_running(false);
    //make a new connection force tcp listener exit accept() blocking
    TcpStream::connect("localhost:3333");
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            if size <= 0 {
                stream.shutdown(Shutdown::Both);
                println!("connection is error");
                false
            } else {
                // echo everything!
                //stream.write(&data[0..size]).unwrap();
                println!("client: {}", String::from_utf8_lossy(&data[0..size]));
                let mut decoder = Decoder::new(BufReader::new(&data[0..size]));
                match decoder.decode() {
                    Ok(requestCmd) => {
                        println!("parse cmd: {:?}", requestCmd);
                        match requestCmd {
                            Value::String(cmd) => {
                                if cmd.trim() == "shutdown" {
                                    println!("Shutting down server ...");
                                    close_server();
                                }
                            },
                            _ => { }
                        }
                    },
                    Err(e) => {
                        println!("parse cmd failed: {:?}", e);
                    }
                }
                true
            }
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    set_server_running(true);
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
        if !is_server_running() {
            break;
        }
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
    println!("Server is shutdown.");
}