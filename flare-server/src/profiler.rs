
use super::sample::*;
use std::{io, thread};
use std::sync::{Arc, Mutex};
use websocket::sync::Server;
use websocket::OwnedMessage;
use websocket::sync::sender::Sender;
use websocket::sender::Writer;
use websocket::server::upgrade::WsUpgrade;
use websocket::server::upgrade::sync::Buffer;
use serde::Serialize;
use utils::*;
use std::collections::HashMap;
use std::io::ErrorKind;
use serde_json::{json, Value};
use std::cmp::{min, max};
use chrono::Local;
use flare_utils::stopwatch::Stopwatch;
use tree::TreeNode;
use inferno::flamegraph::*;
use inferno::flamegraph;
use std::str::FromStr;
use inferno::flamegraph::color::BackgroundColor;
use ::{tree, utils};
use inferno::flamegraph::merge::{TimedFrame, Frame};
use super::http_server::*;
use method_analysis::*;

type JsonValue = serde_json::Value;

pub const FLARE_SAMPLES_DIR : &str = "flare-samples";

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
    sample_session_map: HashMap<String, Arc<Mutex<SampleCollector>>>
}

impl Profiler {
    pub fn new() -> Arc<Mutex<Profiler>> {
        let mut inst = Arc::new(Mutex::new(Profiler {
            self_ref: None,
            bind_addr: "0.0.0.0:3891".to_string(),
            running: true,
            sample_session_map: HashMap::new(),
        }));
        inst.lock().unwrap().self_ref = Some(inst.clone());
        inst.lock().unwrap().init();
        inst
    }

    pub fn init(&mut self) {
        match std::fs::read_dir(FLARE_SAMPLES_DIR) {
            Err(e) => {
                match std::fs::create_dir(FLARE_SAMPLES_DIR) {
                    Err(e) => {
                        println!("create dir failed: {}, error: {:?}", FLARE_SAMPLES_DIR, e);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn connect_agent(&mut self, agent_addr: &str) -> io::Result<String> {
        println!("connecting to agent: {}", agent_addr);
        let instance_id = agent_addr.to_string();
        let value = self.sample_session_map.get(&instance_id);
        if value.is_some() {
            println!("already connected to agent: {}", agent_addr);
            return Ok(instance_id);
        }

        let mut collector = SampleCollector::new(agent_addr)?;
        collector.lock().unwrap().subscribe_events()?;
        println!("connect agent: {} successful", agent_addr);
        self.sample_session_map.insert(instance_id.clone(), collector);
        Ok(instance_id)
    }

    pub fn open_sample(&mut self, sample_data_dir: &str) -> io::Result<String> {
        println!("open sample {} ..", sample_data_dir);
        let instance_id = sample_data_dir.to_string();
        if let Ok(value) = self.get_sample_collector(&instance_id) {
            return Ok(instance_id);
        }

        let mut collector = SampleCollector::open(sample_data_dir)?;
        self.sample_session_map.insert(instance_id.clone(), collector);
        Ok(instance_id)
    }

    pub fn close_session(&mut self, session_id: &str) -> io::Result<()> {
        if let Some(collector) = self.sample_session_map.remove(session_id) {
            println!("close session: {}", session_id);
            collector.lock().unwrap().close();
        }

        Ok(())
    }

    pub fn close_all_session(&mut self) -> io::Result<()> {
        let session_ids = self.sample_session_map.keys().map(|x|{ x.to_string() }).collect::<Vec<String>>();
        for session_id in &session_ids {
            self.close_session(session_id);
        }
        Ok(())
    }

    fn get_sample_collector(&mut self, session_id: &str) -> io::Result<Arc<Mutex<SampleCollector>>> {
        let collector = if let Some(_collector) = self.sample_session_map.get(session_id) {
            Some(_collector.clone())
        }else {
            None
        };

        if let Some(_collector) = collector {
            if _collector.lock().unwrap().is_disconnected() {
                println!("sample session is disconnected: {}, removing it", session_id);
                self.sample_session_map.remove(session_id);
                Err(io::Error::new(ErrorKind::NotFound, "sample session is disconnected"))
            }else {
                Ok(_collector)
            }
        } else {
            Err(io::Error::new(ErrorKind::NotFound, "sample session not found"))
        }
    }

    pub fn get_dashboard(&mut self, session_id: &str) -> io::Result<DashboardInfo> {
        let collector = self.get_sample_collector(session_id)?;
        let data = collector.lock().unwrap().get_dashboard();
        Ok(data)
    }

    pub fn get_all_thread_ids(&mut self, session_id: &str) -> io::Result<Vec<i64>> {
        let collector = self.get_sample_collector(session_id)?;
        let dashboard = collector.lock().unwrap().get_dashboard();
        let mut thread_ids = vec![];
        for thread in &dashboard.threads {
            thread_ids.push(thread.id);
        }
        Ok(thread_ids)
    }

    pub fn get_thread_cpu_times(&mut self, session_id: &str, thread_ids: &[i64], mut start_time: i64, mut end_time: i64, mut unit_time_ms: i64, graph_width: i64) -> io::Result<Vec<Value>> {
        if let Some(collector) = self.sample_session_map.get(session_id) {
            let sample_info = collector.lock().unwrap().get_sample_info();
            //限制时间范围
            if start_time < 0 {
                start_time = sample_info.record_start_time;
            } else {
                start_time = max(start_time, sample_info.record_start_time);
            }

            if end_time < 0 {
                end_time = sample_info.last_record_time;
            } else {
                end_time = min(end_time, sample_info.last_record_time);
            }

            if unit_time_ms < 10 {
                //计算聚合单位时间
                let dt = end_time - start_time;
                if dt <= 0 {
                    return Err(new_invalid_input_error("time period error, end_time must be greater than start_time"))
                }
                let mut ratio = dt / graph_width / sample_info.sample_interval;
                //超过十倍 按照十倍缩放
                if ratio > 10 {
                    ratio = ratio / 10 * 10;
                }
                unit_time_ms = ratio * sample_info.sample_interval;
            }


            let mut thread_cpu_times = vec![];
            for thread_id in thread_ids {
                let ts_result = collector.lock().unwrap().get_thread_cpu_time(thread_id, start_time, end_time, unit_time_ms);
                if let Some(ts_result) = ts_result {
                    let ts_data = ts_result.data.as_int64();
                    thread_cpu_times.push(json!({
                        "id":  thread_id,
                        "start_time": ts_result.begin_time,
                        "end_time": ts_result.end_time,
                        "unit_time_ms": ts_result.unit_time,
                        "total_cpu_time": ts_result.total_cpu_time,
                        "steps": ts_result.steps,
                        "ts_data": ts_data
                    }));
                }
            }
            Ok(thread_cpu_times)
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample session not found"))
        }
    }

    pub fn get_call_tree(&mut self, session_id: &str, thread_ids: &[i64], start_time: i64, end_time: i64) -> io::Result<TreeNode> {
        //xxx
        let collector = self.get_sample_collector(session_id)?;
        let call_tree = collector.lock().unwrap().get_call_tree(thread_ids, start_time, end_time)?;

        //convert to json
        Ok(call_tree.to_tree())
    }

    pub fn create_flame_graph_svg(&mut self, session_id: &str, thread_id: i64, start_time: &mut i64, end_time: &mut i64, stats_type_str: &str, image_width: usize) -> io::Result<String> {
        let mut stats_type = StatsType::DURATION;
        if let Ok(x) = StatsType::from_str(stats_type_str) {
            stats_type = x;
        }else {
            return Err(new_invalid_input_error(&format!("invalid stats_type: {}", stats_type_str)));
        }
        let count_name = match stats_type {
            StatsType::DURATION => "ms",
            StatsType::CPU_TIME => "micros",
            StatsType::SAMPLES => "samples",
        };
        let collector = self.get_sample_collector(session_id)?;
        //create frame graph
        let mut options = flamegraph::Options {
            //colors: Palette::from_str("java").unwrap(),
            //bgcolors: Some(BackgroundColor::from_str("blue").unwrap()),
            //hash: true,
            //top-down flame graph
            direction: Direction::Inverted,
            no_sort: false,
            image_width: Some(image_width),
            count_name: count_name.to_string(),
            ..Default::default()
        };
        let mut writer = vec![];

//        let call_stacks = collector.lock().unwrap().get_collapsed_call_stacks(thread_id, start_time, end_time, stats_type)?;
//        let input = call_stacks.join("\n");
//        if let Err(e) = flamegraph::from_lines(&mut options, input.lines(), &mut writer) {
//            return Err(new_error(ErrorKind::Other, &format!("create flame graph failed: {}", e)));
//        }

        let stack_tree = collector.lock().unwrap().get_sequenced_call_tree(thread_id, start_time, end_time, true)?;
        let mut frames = vec![];
        let mut time = stack_tree.duration as usize;
        let mut delta_max = 0;
        self.prepare_flame_graph_frames(&stack_tree, &mut frames, &mut delta_max);

        if let Err(e) = flamegraph::from_frames(&mut options, &mut writer, &mut frames, time, delta_max) {
            return Err(new_error(ErrorKind::Other, &format!("create flame graph failed: {}", e)));
        }

        match std::str::from_utf8(&writer) {
            Ok(svg) => Ok(svg.to_string()),
            Err(e) => Err(new_error(ErrorKind::Other, &format!("flame graph to string failed: {}", e)))
        }
    }

    fn prepare_flame_graph_frames<'a>(&self, node: &'a Box<TreeNode>, frames: &mut Vec<TimedFrame<'a>>, delta_max: &mut usize) {
        let frame = TimedFrame::new(
            &node.label,
            node.depth as usize,
            node.start_time as usize,
            (node.start_time + node.duration) as usize,
            None
        );
        match frame.end_time.checked_sub(frame.start_time) {
            Some(x) => {},
            None => {
                println!("overflow: {}, time: {} - {}", node.label, frame.end_time, frame.start_time)
            }
        }
        frames.push(frame);
//        if node.children.is_empty() {
//            *time += node.duration as usize;
//        }
        for child in &node.children {
            self.prepare_flame_graph_frames(child, frames,delta_max);
        }
    }

    pub fn get_sequenced_call_tree(&mut self, session_id: &str, thread_id: i64, start_time: &mut i64, end_time: &mut i64, stats_type_str: &str) -> io::Result<Box<tree::TreeNode>> {
        let collector = self.get_sample_collector(session_id)?;
        let result = collector.lock().unwrap().get_sequenced_call_tree(thread_id, start_time, end_time, true);
        result
    }

    pub fn get_sample_info(&mut self, session_id: &str) -> io::Result<SampleInfo> {
        if let Some(collector) = self.sample_session_map.get(session_id) {
            Ok(collector.lock().unwrap().get_sample_info())
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample session not found"))
        }
    }

    fn list_methods_by_filter(&self, session_id: &str, method_name_filter: &str) -> io::Result<Vec<MethodInfo>> {
        if let Some(collector) = self.sample_session_map.get(session_id) {
            Ok(collector.lock().unwrap().list_methods_by_filter(method_name_filter)?)
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample session not found"))
        }
    }

    fn search_slow_method_calls(&self, session_id: &str, thread_id: i64, method_ids: &[i64], min_duration: i64, max_duration: i64) -> io::Result<Vec<Box<MethodCall>>> {
        if let Some(collector) = self.sample_session_map.get(session_id) {
            Ok(collector.lock().unwrap().search_slow_method_calls(thread_id, method_ids, min_duration, max_duration)?)
        }else {
            Err(io::Error::new(ErrorKind::NotFound, "sample session not found"))
        }
    }

    fn start_http_server(&mut self) {
        thread::spawn(|| {
            SimpleHttpServer::start_server();
        });
    }

    fn start_ws_server(&mut self) {
        let self_ref = self.self_ref.as_ref().unwrap().clone();
        let bind_addr = self.bind_addr.clone();
        thread::spawn(move || {
            match Server::bind(bind_addr.clone()) {
                Ok(server) => {
                    println!("Flare profiler started on port: {}", bind_addr);
                    for request in server.filter_map(Result::ok) {
                        if !self_ref.lock().unwrap().is_running() {
                            println!("Shutting down flare analysis server ...");
                            return;
                        }
                        Profiler::handle_connection(self_ref.clone(), request);
                    }
                }
                Err(e) => {
                    println!("Start flare analysis server failed, bind addr: {}, error: {}", bind_addr, e);
                    self_ref.lock().unwrap().shutdown();
                }
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
                        let mut cmd = String::new();
                        if let Err(e) = self_ref.lock().unwrap().handle_request(&mut sender,json.clone(), &mut cmd) {
                            let err = e.to_string();
                            println!("handle request failed: {}, cmd: {}, json: {}", err, cmd, json);
                            //send error
                            sender.send_message(&wrap_error_response(&cmd, &err));
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
            return Err(new_invalid_input_error("missing attribute 'cmd'"));
        }
        _out_cmd.push_str(cmd);

        match cmd {
            "list_sessions" => {
                self.handle_list_sessions(sender, cmd, options)?;
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
            "close_session" => {
                self.handle_close_session_request(sender, cmd, options)?;
            }
            "close_all_session" => {
                self.handle_close_all_session_request(sender, cmd, options)?;
            }
            "dashboard" => {
                self.handle_dashboard_request(sender, cmd, options)?;
            }
            "cpu_time" => {
                self.handle_cpu_time_request(sender, cmd, options)?;
            }
            "call_tree" => {
                self.handle_call_tree_request(sender, cmd, options)?;
            }
            "sequenced_call_tree" => {
                self.handle_sequenced_call_tree_request(sender, cmd, options)?;
            }
            "flame_graph" => {
                self.handle_flame_graph_request(sender, cmd, options)?;
            }
            "list_methods_by_filter" => {
                self.handle_list_methods_by_filter_request(sender, cmd, options)?;
            }
            "search_slow_method_calls" => {
                self.handle_search_slow_method_calls_request(sender, cmd, options)?;
            }
            _ => {
                println!("unknown cmd: {}, request: {}", cmd, json_str);
            }
        }
        Ok(())
    }

    //list open sessions
    fn handle_list_sessions(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut sample_sessions = vec![];
        for (instance_id, collector) in self.sample_session_map.iter() {
            let sample_type = collector.lock().unwrap().get_sample_type();
            sample_sessions.push(json!({"session_id": instance_id, "type": sample_type.to_string()}))
        }
        let data = json!({"sample_sessions": sample_sessions});
        sender.send_message(&wrap_response(cmd, &data));
        Ok(())
    }

    //list history samples
    fn handle_history_samples(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut samples = vec![];
        let paths = std::fs::read_dir(FLARE_SAMPLES_DIR)?;
        for dir in paths {
            let path_buf = dir.unwrap().path();
            if !std::fs::metadata(&path_buf).unwrap().is_dir() {
                continue;
            }
            samples.push(json!({"path": path_buf.to_str(), "type": "file"}));
        }
        let data = json!({"history_samples": samples});
        sender.send_message(&wrap_response(cmd, &data));
        Ok(())
    }

    fn handle_open_sample(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let sample_data_dir = options["sample_data_dir"].as_str().unwrap_or("");
        if sample_data_dir == "" {
            return Err(new_invalid_input_error("missing option 'sample_data_dir'"));
        }
        let instance_id = self.open_sample(sample_data_dir)?;
        sender.send_message(&wrap_response(&cmd, &json!({ "session_id": instance_id, "type": "file" })));
        Ok(())
    }

    fn handle_attach_jvm(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let target_pid = options["target_pid"].as_u64();
        if target_pid.is_none() {
            return Err(new_invalid_input_error("missing option 'target_pid'"));
        }

        let sample_interval_ms = options["sample_interval_ms"].as_u64().unwrap_or(20);
        let sample_duration_sec = options["sample_duration_sec"].as_u64().unwrap_or(0);

        //attach
        Ok(())
    }

    fn handle_connect_agent(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let agent_addr = options.get("agent_addr").map_or(None, |x| x.as_str());
        if agent_addr.is_none() {
            return Err(new_invalid_input_error("missing option 'agent_addr'"));
        }
        let instance_id = self.connect_agent(agent_addr.unwrap())?;
        sender.send_message(&wrap_response(&cmd, &json!({ "session_id": instance_id, "type": "attach" })));

        Ok(())
    }

    fn handle_close_session_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        self.close_session(session_id)?;
        sender.send_message(&wrap_response(&cmd, &json!({ "session_id": session_id})));
        Ok(())
    }

    fn handle_close_all_session_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        self.close_all_session()?;
        sender.send_message(&wrap_response(&cmd, &json!({})));
        Ok(())
    }

    fn handle_dashboard_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        let dashboard_info = self.get_dashboard(session_id)?;
        sender.send_message(&wrap_response(&cmd, &dashboard_info));
        Ok(())
    }

    fn handle_cpu_time_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        let mut thread_ids = get_option_as_int_array(options, "thread_ids")?;
        let start_time = get_option_as_int(options, "start_time", -1);
        let end_time = get_option_as_int(options, "end_time", -1);
        let graph_width = get_option_as_int(options, "graph_width", 900);
        let unit_time_ms = get_option_as_int(options, "unit_time_ms", -1);

        if thread_ids.is_empty() {
            thread_ids = self.get_all_thread_ids(session_id)?;
        }
        //TODO fetch only top n threads data
        //fetch and send in batches, avoid long waiting
        let t0 = Local::now().timestamp_millis();
        println!("[{}] handle_cpu_time_request, fetching thread count: {}", utils::nowTime(), thread_ids.len());
        let mut start = 0;
        while start < thread_ids.len() {
            let t1 = Local::now().timestamp_millis();
            let end = min(start+50, thread_ids.len());
            let thread_cpu_times = self.get_thread_cpu_times(session_id, &thread_ids[start..end], start_time, end_time, unit_time_ms, graph_width)?;;
            let result = json!({
                "session_id": session_id,
                "thread_cpu_times": thread_cpu_times
            });
            let t2 = Local::now().timestamp_millis();
            sender.send_message(&wrap_response(&cmd, &result));
            println!("[{}] fetch thread cpu time data cost: {}ms, threads: {}-{}", utils::nowTime(), t2-t1, start, end);
            start = end;
        }
        let t100 = Local::now().timestamp_millis();
        println!("[{}] handle_cpu_time_request total cost: {}ms, thread count: {}", utils::nowTime(), t100-t0, thread_ids.len());

        Ok(())
    }

    fn handle_call_tree_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        let thread_ids = get_option_as_int_array(options, "thread_ids")?;
        let start_time = get_option_as_int(options, "start_time", -1);
        let end_time = get_option_as_int(options, "end_time", -1);
        let mut sw = Stopwatch::start_new();

        let call_tree = self.get_call_tree(session_id, thread_ids.as_slice(), start_time, end_time)?;
        println!("build call tree data cost: {}ms, threads: {:?}", sw.lap(), &thread_ids);

        let result = json!({
                "session_id": session_id,
                "call_tree_data": [call_tree]
            });
        let message = wrap_response(&cmd, &result);
        println!("wrap message cost: {}ms", sw.lap());

        sender.send_message(&message);
        println!("handle_call_tree_request total cost: {}ms", sw.elapsed_ms());
        Ok(())
    }

    fn handle_flame_graph_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        let thread_id = get_option_as_int(options, "thread_id", -1);
        let start_time = get_option_as_int(options, "start_time", -1);
        let end_time = get_option_as_int(options, "end_time", -1);
        let mut image_width = get_option_as_int(options, "image_width", 900);
        if image_width <= 0 {
            image_width = 900;
        }
        let stats_type = get_option_as_str(options, "stats_type", "duration");
        let mut sw = Stopwatch::start_new();

        if thread_id <= 0 {
            return Err(new_invalid_input_error("missing or invalid option 'thread_id'"));
        }
        let mut new_start_time = start_time;
        let mut new_end_time = end_time;
        let svg = self.create_flame_graph_svg(session_id, thread_id, &mut new_start_time, &mut new_end_time, stats_type, image_width as usize)?;
        let result = json!({
                "session_id": session_id,
                "thread_id": thread_id,
                "start_time": new_start_time,
                "end_time": new_end_time,
                "stats_type": stats_type,
                "image_width": image_width,
                "flame_graph_data": svg
            });
        let message = wrap_response(&cmd, &result);
        sender.send_message(&message);
        println!("handle_flame_graph_request total cost: {}ms", sw.elapsed_ms());

        Ok(())
    }

    fn handle_sequenced_call_tree_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let session_id = get_option_as_str_required(options, "session_id")?;
        let thread_id = get_option_as_int(options, "thread_id", -1);
        let start_time = get_option_as_int(options, "start_time", -1);
        let end_time = get_option_as_int(options, "end_time", -1);
        let stats_type = get_option_as_str(options, "stats_type", "duration");
        let mut sw = Stopwatch::start_new();

        if thread_id <= 0 {
            return Err(new_invalid_input_error("missing or invalid option 'thread_id'"));
        }
        let mut new_start_time = start_time;
        let mut new_end_time = end_time;
        let stacks = self.get_sequenced_call_tree(session_id, thread_id, &mut new_start_time, &mut new_end_time, stats_type)?;
        let result = json!({
                "session_id": session_id,
                "thread_id": thread_id,
                "start_time": new_start_time,
                "end_time": new_end_time,
                "stats_type": stats_type,
                "sequenced_call_tree_data": stacks
            });
        let message = wrap_response(&cmd, &result);
        sender.send_message(&message);
        println!("handle_sequenced_call_tree_request total cost: {}ms", sw.elapsed_ms());

        Ok(())
    }

    fn handle_list_methods_by_filter_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut sw = Stopwatch::start_new();
        let session_id = get_option_as_str_required(options, "session_id")?;
        let method_name_filter = get_option_as_str(options, "method_name_filter", "");

        let method_info_vec = self.list_methods_by_filter(session_id, method_name_filter)?;
        let filter_method_size = method_info_vec.len();
        println!("filter method size: {}", filter_method_size);
        let mut method_infos: &[MethodInfo] = &method_info_vec;
//        if method_infos.len() > 30 {
//            method_infos = &method_info_vec[0..30];
//        }
        let result = json!({
                "session_id": session_id,
                "method_name_filter": method_name_filter,
                "total_method_size": filter_method_size,
                "method_infos": method_infos
            });
        let message = wrap_response(&cmd, &result);
        sender.send_message(&message);
        println!("handle_list_methods_by_filter_request total cost: {}ms", sw.elapsed_ms());
        Ok(())
    }

    fn handle_search_slow_method_calls_request(&mut self, sender: &mut Writer<std::net::TcpStream>, cmd: &str, options: &serde_json::Map<String, serde_json::Value>) -> io::Result<()> {
        let mut sw = Stopwatch::start_new();
        let session_id = get_option_as_str_required(options, "session_id")?;
        let mut method_ids = get_option_as_int_array(options, "method_ids")?;
        let min_duration = get_option_as_int(options, "min_duration", 100);
        let max_duration = get_option_as_int(options, "max_duration", -1);
        let max_size = get_option_as_int(options, "max_size", 2000) as usize;
        let thread_name_filter = get_option_as_str(options, "thread_name_filter", "");

        //sort asc for binarysearch
        method_ids.sort();

        //let method_calls : Vec<String> = vec![];
        let mut search_error = false;
        let mut search_error_msg = "".to_string();
        if let Some(collector) = self.sample_session_map.get(session_id) {
            let mut total = 0;
            let threads = collector.lock().unwrap().get_threads()?;
            let thread_size = threads.len();
            let mut search_progress = 0;
            let mut method_analysis = MethodAnalysis::new();
            //search every thread
            for (i,thread) in threads.iter().enumerate() {
                sw.lap();

                //filter thread by name
                if !thread_name_filter.is_empty() && !thread.name.contains(thread_name_filter) {
                    continue;
                }

                match collector.lock().unwrap().search_slow_method_calls(thread.id, &method_ids, min_duration, max_duration) {
                    Ok(method_calls ) => {
                        let search_cost = sw.lap();
                        //send progress
                        let new_search_progress = 100*i/thread_size;
                        if new_search_progress - search_progress > 5 {
                            search_progress = new_search_progress;
                            let result = json!({
                                "session_id": session_id,
                                //"method_ids": method_ids,
                                "search_progress": search_progress,
                                "search_finished": false,
                                "search_message": format!("searching {}", thread.name)
                            });
                            sender.send_message(&wrap_response(&cmd, &result));
                            println!("search progress: {}%", search_progress);
                        }

                        if method_calls.is_empty() {
                            //println!("slow method calls not found, thread: {}, search cost: {}ms", thread.id, search_cost);
                            continue;
                        }
                        total += method_calls.len();
                        println!("found slow method calls: {}, thread: {}, search cost: {}ms, send cost: {}ms", method_calls.len(), thread.id,  search_cost, sw.lap());

                        for method_call in method_calls {
                            method_analysis.add_method_call(method_call);
                        }
                    }
                    Err(e) => {
                        println!("found slow method calls failed, thread: {}, error: {}", thread.id, e);
                    }
                }

                if total >= max_size {
                    search_error = true;
                    search_error_msg = format!("too many slow method calls: {}, search aborted!", total);
                    println!("too many slow method calls: {}, search progress: {}/{} ({}%), abort searching!", total, i, thread_size, search_progress);
                    break;
                }
            }

            //fill call method name
            for method_group in &mut method_analysis.call_groups {
                for call in &mut method_group.call_stack {
                    match collector.lock().unwrap().get_method_info(call.method_id) {
                        Some(val) => {
                            call.full_name = val.full_name.clone();
                        },
                        None => {
                            call.full_name = call.method_id.to_string();
                        }
                    }
                }
            }

            let result = json!({
                "session_id": session_id,
                "method_ids": method_ids,
                "method_call_groups": method_analysis.get_method_groups(),
                "search_progress": search_progress,
                "search_finished": !search_error,
                "search_error": search_error,
                "search_message": search_error_msg
            });
            sender.send_message(&wrap_response(&cmd, &result));
            println!("handle_search_slow_method_calls_request total cost: {}ms", sw.elapsed_ms());
        }else {
            return Err(io::Error::new(ErrorKind::NotFound, "sample session not found"));
        }
        Ok(())
    }

    pub fn startup(&mut self) {
        self.start_ws_server();
        self.start_http_server();
    }

    pub fn shutdown(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}