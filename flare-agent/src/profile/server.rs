use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use resp::{Value, Decoder};
use std::io::BufReader;
use std::collections::HashMap;
use std::sync::{Mutex,Arc,RwLock};
use std::collections::VecDeque;
use super::sample::ThreadData;
use profile::encoder::*;
use profile::sample::*;

lazy_static! {
    static ref RUNNING_SERVER: Mutex<bool> = Mutex::new(false);
    static ref DATA_QUEUE: Mutex<SampleQueue>  = Mutex::new(SampleQueue::new());
}

pub fn add_sample_data(sample_data: Box<SampleData + Send>) {
    let mut data_queue = DATA_QUEUE.lock().unwrap();
    let mut queue = &mut data_queue.queue;
    queue.push_back(sample_data);
    while(queue.len() > 512){
        queue.pop_front();
    }
}

pub fn add_sample_data_batch(data_vec: Vec<Box<SampleData + Send>>) {
    let mut data_queue = DATA_QUEUE.lock().unwrap();
    let mut queue = &mut data_queue.queue;
    for sample_data in data_vec {
        queue.push_back(sample_data);
    }
    while(queue.len() > 512){
        queue.pop_front();
    }
}

fn set_server_running(val: bool) {
    let mut a = RUNNING_SERVER.lock().unwrap();
    *a = val;
}

pub fn is_server_running() -> bool {
    *RUNNING_SERVER.lock().unwrap()
}

pub fn stop_server() {
    set_server_running(false);
    //make a new connection force tcp listener exit accept() blocking
    TcpStream::connect("localhost:3333");
}

pub fn start_server() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Flare agent server listening on port 3333");
    set_server_running(true);
    for stream in listener.incoming() {
        if !is_server_running() {
            println!("Flare agent server is not running, exiting");
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
    println!("Flare agent server is shutdown.");
}

fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 1024]; // using 1024 byte buffer
    while match stream.read(&mut data) {
        Ok(size) => {
            let clientRequest = parse_request(&data[0..size]);
            //dispatch request
            dispatch_request(&mut stream, &clientRequest);

            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn dispatch_request(stream: &mut TcpStream, clientRequest: &Value) {
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
            "resume-sample" => {
                handle_resume_sample_cmd(stream, &cmd_options);
            },
            "pause-sample" => {
                handle_pause_sample_cmd(stream, &cmd_options);
            },
            "stop-sample" => {
                handle_stop_sample_cmd(stream, &cmd_options);
            },
            "subscribe-events" => {
                handle_subscribe_events_cmd(stream, &cmd_options);
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

fn handle_resume_sample_cmd(stream: &mut TcpStream, cmd_options: &HashMap<String, Value>) {
    //resume
}

fn handle_pause_sample_cmd(stream: &mut TcpStream, cmd_options: &HashMap<String, Value>) {
    //pause
}

fn handle_stop_sample_cmd(stream: &mut TcpStream, cmd_options: &HashMap<String, Value>) {
    stop_server();
}

fn handle_subscribe_events_cmd(stream: &mut TcpStream, cmd_options: &HashMap<String, Value>) {
    println!("subscribe event loop start");
    loop {
        if let Some(sample_data) = DATA_QUEUE.lock().unwrap().queue.pop_front() {
            //encode and send sample data
            let buf = sample_data.encode();
            if let Err(e) = stream.write_all(buf.as_slice()) {
                println!("write sample data failed: {}", e);
                break;
            }
        }else {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
    println!("subscribe event loop exit")
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

