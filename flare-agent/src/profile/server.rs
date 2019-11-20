use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use resp::{Value, Decoder};
use std::io::BufReader;
use std::collections::HashMap;
use std::sync::{Mutex, Arc, RwLock, mpsc};
use std::collections::VecDeque;
use super::sample::ThreadData;
use profile::encoder::*;
use profile::sample::*;
use std::time::Duration;

lazy_static! {
    static ref DATA_QUEUE: Mutex<SampleQueue>  = Mutex::new(SampleQueue::new());
    static ref SAMPLE_SERVER: Mutex<SampleServer>  = Mutex::new(SampleServer::new());
}

pub struct SampleServer {
    sample_interval: u64,
    start_time: i64,
    running: bool,
    bind_port: u16,
    bind_host: String,
    sender: Option<mpsc::Sender<resp::Value>>,
    receiver: Option<mpsc::Receiver<resp::Value>>,
}

impl SampleServer {
    pub fn new() -> SampleServer {
        SampleServer {
            sample_interval: 0,
            start_time: 0,
            running: false,
            bind_port: 3333,
            bind_host: "0.0.0.0".to_string(),
            sender: None,
            receiver: None,
        }
    }

    pub fn set_options(&mut self, sender: mpsc::Sender<resp::Value>, receiver: mpsc::Receiver<resp::Value>, start_time: i64, sample_interval: u64) {
        self.start_time = start_time;
        self.sample_interval = sample_interval;
        self.sender = Some(sender);
        self.receiver = Some(receiver);
    }

    pub fn set_running(&mut self, val: bool) {
        self.running = val;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn send_request(&self, request: resp::Value) {
        if let Some(tx) = &self.sender {
            tx.send(request);
        }
    }

    pub fn recv_response(&self) -> Option<resp::Value> {
        if let Some(rx) = &self.receiver {
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(val) => {
                    return Some(val);
                },
                Err(e) => {
                    //println!("recv response from sample thread failed: {:?}", e);
                },
            }
        }
        None
    }
}

pub fn get_server() -> &'static Mutex<SampleServer> {
    &SAMPLE_SERVER
}

pub fn add_sample_data(sample_data: Box<SampleData + Send>) {
    let mut data_queue = DATA_QUEUE.lock().unwrap();
    let mut queue = &mut data_queue.queue;
    queue.push_back(sample_data);
    while(queue.len() > 10000){
        queue.pop_front();
    }
}

pub fn add_sample_data_batch(data_vec: Vec<Box<SampleData + Send>>) {
    let mut data_queue = DATA_QUEUE.lock().unwrap();
    data_queue.push_back(data_vec);
}

fn set_server_running(val: bool) {
    SAMPLE_SERVER.lock().unwrap().set_running(val);
}

pub fn is_server_running() -> bool {
    SAMPLE_SERVER.lock().unwrap().is_running()
}

pub fn stop_server() {
    set_server_running(false);
    //make a new connection force tcp listener exit accept() blocking
    TcpStream::connect("localhost:3333");
}

pub fn start_server() {
    let timer = timer::Timer::new();
    let guard = {
        timer.schedule_repeating(chrono::Duration::milliseconds(3000), move || {
            DATA_QUEUE.lock().unwrap().stats();
        })
    };

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Flare agent server listening on port 3333");
    set_server_running(true);
    let mut last_client_stream: Option<TcpStream> = None;
    for stream in listener.incoming() {
        if !is_server_running() {
            println!("Flare agent server is not running, exiting");
            break;
        }
        match stream {
            Ok(stream) => {
                //close prev connectiopn
                if let Some(last_stream) = &last_client_stream {
                    let mut peer_addr = "??".to_string();
                    match last_stream.peer_addr() {
                        Err(e) => {
                            println!("Get prev connection  peer_addr failed: {}", e);
                        },
                        Ok(addr) => {
                            println!("Flare agent is already connected to collector: {} ...", addr);
                            peer_addr = addr.to_string();
                        }
                    }
                    //check prev connection error
                    match last_stream.take_error() {
                        Ok(x) => {
                            if let Some(e) = x {
                                println!("Prev connection is error: {}", e);
                            }else {
                                println!("Closing prev connection: {} ...", peer_addr);
                                last_stream.shutdown(Shutdown::Both);
                            }
                        },
                        Err(e) => {
                            println!("Get prev connection status failed: {}", e);
                        }
                    }
                } else {
                    println!("Flare agent is idle.")
                }
                last_client_stream = None;
                println!("New connection: {}", stream.peer_addr().unwrap());

                //save last connection
                match stream.try_clone() {
                    Ok(stream_copy) => {
                        last_client_stream = Some(stream_copy);
                    },
                    Err(e) => {
                        println!("Clone stream failed: {}", e);
                    }
                }

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
    drop(guard);
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
        Err(e) => {
            println!("An error occurred, terminating connection with {}, error: {}", stream.peer_addr().unwrap(), e);
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

pub fn parse_request_options(request: &Vec<Value>) -> HashMap<String, Value> {
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

    //send sample info
//    let start_time = SAMPLE_SERVER.lock().unwrap().start_time;
//    let sample_interval = SAMPLE_SERVER.lock().unwrap().sample_interval;
//    let buf = resp_encode_sample_info(start_time, sample_interval);
//    if let Err(e) = stream.write_all(buf.as_slice()) {
//        println!("send sample info failed: {}", e);
//        return;;
//    }

    //send sample info
    println!("sending sample info to new client ..");
    let request = Value::Array(vec![
        Value::String("get_sample_info".to_string()),
    ]);
    SAMPLE_SERVER.lock().unwrap().send_request(request);
    if let Some(response) = SAMPLE_SERVER.lock().unwrap().recv_response() {
        if let Err(e) = stream.write_all(response.encode().as_slice()) {
            println!("send sample info failed: {}", e);
            return;
        }
    }else {
        println!("recv get_sample_info result failed, stopping subscribe event")
    }

    //send current method cache
    println!("sending method cache to new client ..");
    let request = Value::Array(vec![
        Value::String("get_method_cache".to_string()),
    ]);
    SAMPLE_SERVER.lock().unwrap().send_request(request);
    //transmit method cache
    let mut method_count = 0;
    while let Some(response) = SAMPLE_SERVER.lock().unwrap().recv_response() {
        if let Err(e) = stream.write_all(response.encode().as_slice()) {
            println!("send method cache failed: {}", e);
            //return; //recv all message in channel
        }
        method_count += 1;
    }
    println!("total sent method cache: {}", method_count);

//    println!("recv get_sample_info result failed, stopping subscribe event")

    println!("loop transmit data new client ..");
    let mut sent = false;
    loop {
        //auto release lock while exit guard block
        {
            if let Some(sample_data) = DATA_QUEUE.lock().unwrap().pop_front() {
                sent = true;
                //encode and send sample data
                let buf = sample_data.encode();
                if let Err(e) = stream.write_all(buf.as_slice()) {
                    println!("write sample data failed: {}", e);
                    break;
                }
            }else {
                sent = false;
            }
        }
        if !sent {
            std::thread::sleep(std::time::Duration::from_millis(10));
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

