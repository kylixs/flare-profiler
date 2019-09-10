
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
use serde_json::json;

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
                        let mut _cmd = String::new();
                        if let Err(e) = self_ref.lock().unwrap().handle_request(&mut sender,json, &mut _cmd) {
                            //send error
                            sender.send_message(&wrap_error_response(&_cmd, &e.to_string()));
                        }
                    }
                    _ => {
                        sender.send_message(&message).unwrap()
                    },
                }
            }
        });
    }

    fn handle_request(&mut self, sender: &mut Writer<std::net::TcpStream>, json_str: String, _out_cmd: &mut String) -> io::Result<()> {
        println!("recv: {}", json_str);
        //TODO parse request to json
        let request: JsonValue = serde_json::from_str(&json_str)?;
        let temp;
        let mut options = request["options"].as_object();
        if options.is_none() {
            temp = serde_json::Map::new();
            options = Some(&temp);
        }
        let options = options.unwrap();

        //cmd
        let cmd= request["cmd"].as_str().unwrap_or("");
        if cmd == "" {
            return new_invalid_input_error("missing attribute 'cmd'");
        }
        _out_cmd.push_str(cmd);

        match cmd {
            "list_instances" => {
                self.handle_list_instances(sender, cmd, options)?;
            }
            "history_samples" => {
                self.handle_history_samples(sender, cmd, options)?;
            }
            "open_sample" => {
                self.handle_open_sample(sender, cmd, options)?;
            }
            "attach_jvm" => {
                self.handle_attach_jvm(sender, cmd, options)?;
            }
            "connect_agent" => {
                self.handle_connect_agent(sender, cmd, options)?;
            }
            "dashboard" => {
                self.handle_dashboard_request(sender, cmd, options)?;
            }
            _ => {
                println!("unknown cmd: {}, request: {}", cmd, json_str);
            }
        }
        Ok(())
    }

    //list open instances
    fn handle_list_instances(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut sample_instances = vec![];
        for (instance_id, client) in self.sample_instance_map.iter() {
            let client_type = client.lock().unwrap().get_sample_type();
            sample_instances.push(json!({"instance_id": instance_id, "type": client_type.to_string()}))
        }
        let data = json!({"sample_instances": sample_instances});
        sender.send_message(&wrap_response(cmd, &data));
        Ok(())
    }

    //list history samples
    fn handle_history_samples(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut sample_instances = vec![];
        let paths = std::fs::read_dir("flare-samples")?;
        for path in paths {

            sample_instances.push(json!({"instance_id": path.unwrap().file_name().into_string().unwrap(), "type": "file"}));
        }
        let data = json!({"sample_instances": sample_instances});
        sender.send_message(&wrap_response(cmd, &data));
        Ok(())
    }

    fn handle_open_sample(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let sample_data_dir = options["sample_data_dir"].as_str();
        if sample_data_dir.is_none() {
            return new_invalid_input_error("missing option 'sample_data_dir'");
        }
        //
        Ok(())
    }

    fn handle_attach_jvm(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let target_pid = options["target_pid"].as_u64();
        if target_pid.is_none() {
            return new_invalid_input_error("missing option 'target_pid'");
        }

        let sample_interval_ms = options["sample_interval_ms"].as_u64().unwrap_or(20);
        let sample_duration_sec = options["sample_duration_sec"].as_u64().unwrap_or(0);

        //attach
    }

    fn handle_connect_agent(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let agent_addr = options["agent_addr"].as_str();
        if agent_addr.is_none() {
            return new_invalid_input_error("missing option 'agent_addr'");
        }
        self.connect_agent(agent_addr.unwrap());
        Ok(())
    }

    fn handle_dashboard_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let sample_instance = get_option_required_as_str(options, "sample_instance")?;
        let dashboard_info = self.get_dashboard(sample_instance)?;
        sender.send_message(&wrap_response(&cmd, &dashboard_info));
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