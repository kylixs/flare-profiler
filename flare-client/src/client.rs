
use super::sampler_client::*;
use std::{io, thread};
use std::sync::{Arc, Mutex};
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::sender::Sender;
use websocket::sender::Writer;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::upgrade::sync::Buffer;
use serde::Serialize;
use client_utils::*;
use std::collections::HashMap;
use std::io::ErrorKind;

type JsonValue = serde_json::Value;

#[derive(Clone, Serialize)]
pub struct FlareResponse<T: ?Sized> {
    pub result: String,
    pub cmd: String,
    pub data: Box<T>
}

pub struct Profiler {
    self_ref: Option<Arc<Mutex<Profiler>>>,
    bind_addr: String,
    running: bool,
    sample_instance_map: HashMap<String, Arc<Mutex<SamplerClient>>>
}

impl Profiler {
    pub fn new() -> Arc<Mutex<Profiler>> {
        let mut inst = Arc::new(Mutex::new(Profiler {
            self_ref: None,
            bind_addr: "0.0.0.0:3344".to_string(),
            running: true,
            sample_instance_map: HashMap::new(),
        }));
        inst.lock().unwrap().self_ref = Some(inst.clone());
        inst
    }

    pub fn connect_agent(&mut self, agent_addr: &str) -> io::Result<()> {
        println!("connecting to agent: {}", agent_addr);
        let mut client = SamplerClient::new(agent_addr)?;
        self.sample_instance_map.insert(agent_addr.to_string(), client.clone());

        thread::spawn(move|| {
            client.lock().unwrap().subscribe_events();
        });

        Ok(())
    }

    pub fn get_dashboard(&mut self, sample_instance: &str) -> io::Result<DashboardInfo> {
        if let Some(client) = self.sample_instance_map.get(sample_instance) {
            Ok(client.lock().unwrap().get_dashboard())
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample instance not found"))
        }
    }

    pub fn get_sample_info(&mut self, sample_instance: &str) -> io::Result<SampleInfo> {
        if let Some(client) = self.sample_instance_map.get(sample_instance) {
            Ok(client.lock().unwrap().get_sample_info())
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample instance not found"))
        }
    }

    fn start_ws_server(&mut self) {
        let self_ref = self.self_ref.as_ref().unwrap().clone();
        let bind_addr = self.bind_addr.clone();
        thread::spawn(move || {
            println!("analysis server binding: {}", bind_addr);
            let server = Server::bind(bind_addr).unwrap();
            for request in server.filter_map(Result::ok) {
                if !self_ref.lock().unwrap().is_running() {
                    println!("Shutting down analysis ws server ...");
                    return;
                }
                Profiler::handle_connection(self_ref.clone(), request);
            }
        });
    }

    fn handle_connection(self_ref: Arc<Mutex<Profiler>>, request: WsUpgrade<std::net::TcpStream, Option<Buffer>>) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let ws_protocol = "flare-profiler";
            if !request.protocols().contains(&ws_protocol.to_string()) {
                request.reject().unwrap();
                return;
            }
            let mut client = request.use_protocol(ws_protocol).accept().unwrap();
//            let mut client = request.accept().unwrap();

            let ip = client.peer_addr().unwrap();
            println!("Connection from {}", ip);

            //send first message
//            let sample_info = self_ref.lock().unwrap().get_sample_info()?;
//            client.send_message(&wrap_response( "sample_info", &sample_info));

            //recv first message
//            client.recv_message();

            //recv and dispatch message
            let (mut receiver, mut sender) = client.split().unwrap();
            for message in receiver.incoming_messages() {
                let message = message.unwrap();
                match message {
                    OwnedMessage::Close(_) => {
                        let message = OwnedMessage::Close(None);
                        sender.send_message(&message).unwrap();
                        println!("Client {} disconnected", ip);
                        return;
                    }
                    OwnedMessage::Ping(ping) => {
                        let message = OwnedMessage::Pong(ping);
                        sender.send_message(&message).unwrap();
                    }
                    OwnedMessage::Text(json) => {
                        Profiler::handle_request(self_ref.clone(), &mut sender,json);
                    }
                    _ => sender.send_message(&message).unwrap(),
                }
            }
        });
    }


    fn handle_request(self_ref: Arc<Mutex<Profiler>>, sender: &mut Writer<std::net::TcpStream>, json_str: String) -> io::Result<()> {
        println!("recv: {}", json_str);
//        let message = OwnedMessage::Text(json_str);
//        sender.send_message(&message);

        //TODO parse request to json
        let request: JsonValue = serde_json::from_str(&json_str)?;
        let temp;
        let options = (
            if let Some(s) = request["options"].as_object() { s
            }else { temp = serde_json::Map::new(); &temp }
        );

        if let JsonValue::String(cmd) = &request["cmd"] {
            match cmd.as_str() {
                "list_instances" => {
                    //
                }
                "history_samples" => {
                    //
                }
                "open_sample" => {
                    let sample_data_dir = options["sample_data_dir"].as_str();
                    if sample_data_dir.is_none() {
                        return new_invalid_input_error("missing option 'sample_data_dir'");
                    }
                    //
                }
                "attach_jvm" => {
                    let sample_method = options["sample_method"].as_str();
                    if sample_method.is_none() {
                        return new_invalid_input_error("missing option 'sample_method'");
                    }

                    let target_pid = options["target_pid"].as_u64();
                    if target_pid.is_none() {
                        return new_invalid_input_error("missing option 'target_pid'");
                    }

                    let sample_interval_ms = options["sample_interval_ms"].as_u64();
                    if sample_interval_ms.is_none() {
                        return new_invalid_input_error("missing option 'sample_interval_ms'");
                    }

                    let mut sample_duration_sec = options["sample_duration_sec"].as_u64();
                    if sample_duration_sec.is_none() {
                        sample_duration_sec = Some(0);
                    }
                    //attach
                }
                "connect_agent" => {
                    let agent_addr = options["agent_addr"].as_str();
                    if agent_addr.is_none() {
                        return new_invalid_input_error("missing option 'agent_addr'");
                    }
                    self_ref.lock().unwrap().connect_agent(agent_addr.unwrap());
                }
                "dashboard" => {
                    if let Some(sample_instance) = options["sample_instance"].as_str() {
                        let dashboard_info = self_ref.lock().unwrap().get_dashboard(sample_instance)?;
                        sender.send_message(&wrap_response(&cmd, &dashboard_info));
                    } else {
                        println!("missing 'sample_instance': {}, request: {}", cmd, json_str);
                    }
                }
                _ => {
                    println!("unknown cmd: {}, request: {}", cmd, json_str);
                }
            }
        }else {
            println!("invalid request: {}", json_str);
        }

        Ok(())
    }

    pub fn startup(&mut self) {
        self.start_ws_server();
    }

    pub fn shutdown(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}