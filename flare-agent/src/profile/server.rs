use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use resp::{Value, Decoder};
use std::io::BufReader;
use std::collections::HashMap;

pub fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");
    for stream in listener.incoming() {
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
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 1024]; // using 1024 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            let clientRequest = parse_request(&data[0..size]);
            //dispatch request
            dispatch_request(&clientRequest);

            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn dispatch_request(clientRequest: &Value) {
    //extract cmd string
    let cmd_vec_result = match clientRequest {
        Value::Array(vec) => {
            let first = &vec[0];
            match first {
                Value::String(s) => {
                    let cmd_options = parse_request_options(vec);
                    Some((s, cmd_options))
                },
                _ => {
                    println!("invalid request array, first element must be String, but get {:?}", first);
                    None
                }
            }
        },
        _ => {
            println!("invalid request, must be an resp array like [String, args1, args2..], but get {:?}", clientRequest);
            None
        }
    };

    //dispatch by cmd str
    if let Some((cmd, cmd_options)) = cmd_vec_result {
        match cmd.as_str() {
            "start-sample" => {
                handle_start_sample_cmd(&cmd_options);
            },
            "stop-sample" => {
                handle_stop_sample_cmd(&cmd_options);
            },
            "subscribe-events" => {
                handle_subscribe_events_cmd(&cmd_options);
            },
            _ => { println!("unknown request cmd: {}, options: {:?}", cmd, cmd_options); }
        }
    }
}

fn parse_request_options(request: &Vec<Value>) -> HashMap<String, Value> {
    let mut result = HashMap::new();
    let mut i = 1;
    while i < request.len()-1 {
        let key = &request[i];
        let value = &request[i+1];
        match key {
            Value::String(str) => {
                result.insert(str.to_string(), value.clone());
            },
            _ => {
                println!("invalid cmd option key, expect String but get: {:?}", key);
            }
        }
        i+=2;
    }
    result
}

fn handle_start_sample_cmd(cmd_options: &HashMap<String, Value>) {

}

fn handle_stop_sample_cmd(cmd_options: &HashMap<String, Value>) {

}

fn handle_subscribe_events_cmd(cmd_options: &HashMap<String, Value>) {

}

fn parse_request(buf: &[u8]) -> Value {
    // echo everything!
    //stream.write(&data[0..size]).unwrap();
    //println!("client: {}", String::from_utf8_lossy(&data[0..size]));
    let mut decoder = Decoder::new(BufReader::new(buf));
    let clientRequest = decoder.decode().unwrap();
    println!("request: {:?}", clientRequest);
    clientRequest
}

