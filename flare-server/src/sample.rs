
use std::collections::*;
//use call_tree::{TreeArena, NodeId};
use std::collections::hash_map::Entry;
use time::Duration;
use std::net::{TcpStream, ToSocketAddrs, Shutdown};
use resp::Value;
use std::io::{Write, Read, BufReader, Error, ErrorKind};
use std::str::from_utf8;
use std::io;
use chrono::Local;
use flare_utils::timeseries::*;
use flare_utils::tuple_indexed::*;
use flare_utils::tuple_indexed::{TupleIndexedFile, TupleValue};
use flare_utils::timeseries::{TimeSeries, TSValue, TimeSeriesFileWriter, TimeSeriesFileReader};
use flare_utils::{ValueType, file_utils};
use std::path::PathBuf;
use utils::*;
use std::hash::Hash;
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;
use flare_utils::file_utils::open_file;
use call_tree::*;
use std::ops::{Index, Deref, DerefMut};
use flare_utils::stopwatch::*;
use std::str::FromStr;
use tree;


type JavaLong = i64;
type JavaMethod = i64;

pub const FLARE_SAMPLES_DIR : &str = "flare-samples";

#[derive(Clone, Serialize, Deserialize)]
pub struct ThreadData {
    pub id: JavaLong,
    pub name: String,
    pub priority: u32,
    pub daemon: bool,
    pub state: String,
    pub cpu_time: i64,
    pub cpu_time_delta: i64,
    pub sample_time: i64,
    #[serde(default = "default_sample_count")]
    pub sample_count: i64,
    pub stacktrace: Vec<i64>,

    //dynamic calc attrs
    #[serde(default)]
    pub duration: i64,
    #[serde(default)]
    pub self_duration: i64,
    #[serde(default)]
    pub self_cpu_time: i64
}

fn default_sample_count() -> i64 {
    1
}

#[derive(Clone)]
pub struct MethodInfo {
    pub method_id: i64,
    pub full_name: String,
    pub hits_count: u32
//    pub source_file: String,
//    pub line_num: u16
}

#[derive(Serialize, Deserialize)]
pub struct DashboardInfo {
    pub sample_info: SampleInfo,
    pub threads: Vec<ThreadData>
    //jvm_info: JvmInfo,
}

#[derive(Serialize, Deserialize)]
pub struct SampleInfo {
    pub sample_interval: i64,
    pub sample_start_time: i64,
    pub record_start_time: i64,
    pub last_record_time: i64,
    pub agent_addr: String,
    pub sample_data_dir: String,
}

#[derive(Serialize, Deserialize)]
pub struct SummaryInfo {
    sample_info: SampleInfo,
    threads: Vec<ThreadData>
}

// 统计方式
#[derive(Eq, PartialEq, Debug, EnumString)]
pub enum StatsType {
    #[strum(serialize="duration")]
    DURATION,

    #[strum(serialize="cpu_time")]
    CPU_TIME,

    #[strum(serialize="samples")]
    SAMPLES,
}

pub struct SampleCollector {
    //self ref
    this_ref: Option<Arc<Mutex<SampleCollector>>>,
    connected: bool,
    disconnected: bool,
    agent_addr: String,
    agent_stream: Option<TcpStream>,
    readonly: bool,
    running: bool,

    //sample option
    sample_interval: i64,
    sample_start_time: i64,
    sample_type: String,

    //collector
    record_start_time: i64,
    last_record_time: i64,
    //last save summary time
    last_save_time: i64,

    //sample data processor
    threads : HashMap<JavaLong, ThreadData>,
    sample_data_dir: String,
    sample_cpu_ts_map: HashMap<JavaLong, Option<Box<TimeSeries+Send>>>,
    sample_cpu_ts_cache: HashMap<String, Option<Arc<TSResult>>>,
    sample_stacktrace_map: HashMap<JavaLong, Option<TupleIndexedFile>>,
    sample_method_idx_file: Option<TupleIndexedFile>,
    method_cache: HashMap<JavaMethod, Option<MethodInfo>>,
//    tree_arena: TreeArena
}

impl SampleCollector {


    pub fn new(addr: &str) -> io::Result<Arc<Mutex<SampleCollector>>> {
        let mut collector = SampleCollector::new_instance();
        collector.lock().unwrap().agent_addr = addr.to_string();
        Ok(collector)
    }

    pub fn open(sample_dir: &str) -> io::Result<Arc<Mutex<SampleCollector>>> {
        println!("load sample data from dir: {}", sample_dir);
        let mut collector = SampleCollector::new_instance();
        match collector.lock().unwrap().load_sample(sample_dir) {
            Ok(_) => {},
            Err(e) => {
                println!("load sample failed: {:?}", e);
                collector.lock().unwrap().close();
                return Err(e);
            }
        }
        Ok(collector)
    }

    fn new_instance() -> Arc<Mutex<SampleCollector>> {
        let mut collector = Arc::new(Mutex::new(SampleCollector {
            this_ref: None,
            readonly: false,
            running: true,
            sample_type: "".to_string(),
            sample_interval: 20,
            sample_start_time: 0,
            record_start_time: 0,
            last_record_time: 0,
            last_save_time: 0,
            threads: HashMap::new(),
            sample_data_dir: "".to_string(),
            sample_cpu_ts_map: HashMap::new(),
            sample_cpu_ts_cache: Default::default(),
            sample_stacktrace_map: HashMap::new(),
            sample_method_idx_file: None,
            connected: false,
            disconnected: false,
            agent_addr: "".to_string(),
            agent_stream: None,
            method_cache: HashMap::new(),
//            tree_arena: TreeArena::new()
        }));
        //self ref for threads
        collector.lock().unwrap().this_ref = Some(collector.clone());
        collector
    }

    pub fn close(&mut self) {
//        if self.running {
//        }
        self.running = false;
        //release self ref 必须释放自引用，否则不会释放此对象，打开的文件句柄也不会自动关闭
        self.this_ref = None;
        //close agent connection
        if let Some(stream) = &self.agent_stream {
            let mut peer_addr = "??".to_string();
            if let Ok(addr) = stream.peer_addr() {
                peer_addr = addr.to_string();
            }
            println!("closing agent connection: {} ..", peer_addr);
            stream.shutdown(Shutdown::Both);
        }
        self.agent_stream = None;
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    //加载取样数据
    fn load_sample(&mut self, sample_data_dir: &str) -> io::Result<()> {
        self.readonly = true;
        self.sample_type = "file".to_string();
        self.sample_data_dir = sample_data_dir.to_string();
        let dir_meta = std::fs::metadata(sample_data_dir);
        if dir_meta.is_err() {
            return Err(new_error(ErrorKind::NotFound, "sample data dir not found"));
        }
        if !dir_meta.unwrap().is_dir() {
            return Err(new_error(ErrorKind::NotFound, "sample data dir is not a directory"));
        }

        self.sample_data_dir = sample_data_dir.to_string();

        //summary info
        let path = format!("{}/summary_info.json", sample_data_dir);
        let json = std::fs::read_to_string(path)?;

        let summary = serde_json::from_str::<SummaryInfo>(&json);
        if summary.is_err() {
            println!("load summary info json failed: {}", summary.err().unwrap());
            return Err(new_error(ErrorKind::InvalidData, "load summary info json failed"));
        }
        let summary = summary.unwrap();

        let sample_info = &summary.sample_info;
        self.sample_start_time = sample_info.sample_start_time;
        self.sample_interval = sample_info.sample_interval;
        self.agent_addr = sample_info.agent_addr.clone();
        self.record_start_time = sample_info.record_start_time;
        self.last_record_time = sample_info.last_record_time;

        //threads
        for thread in &summary.threads {
            self.threads.insert(thread.id, thread.clone());

            //load cpu time ts
            let thread_cpu_ts_file = format!("{}/thread_{}_cpu_time", sample_data_dir, thread.id);
            match TimeSeriesFileReader::new(&thread_cpu_ts_file) {
                Ok(ts) => {
                    self.sample_cpu_ts_map.insert(thread.id, Some(Box::new(ts)));
                },
                Err(e) => {
                    println!("load thread cpu time file failed: {}, err: {}", thread_cpu_ts_file, e);
                }
            }

            //load thread stacktrace
            let thread_stack_file = format!("{}/thread_{}_stack", sample_data_dir, thread.id);
            match TupleIndexedFile::new_reader(&thread_stack_file) {
                Ok(file) => {
                    self.sample_stacktrace_map.insert(thread.id, Some(file));
                },
                Err(e) => {
                    println!("load thread stacktrace file failed: {}, err: {}", thread_stack_file, e);
                }
            }
        }

        //method info idx file
        let method_idx_path = format!("{}/method_info", sample_data_dir);
        let mut method_idx_file = TupleIndexedFile::new_writer(&method_idx_path, ValueType::INT64)?;
        self.sample_method_idx_file = Some(method_idx_file);

        //load threads
//        let paths = std::fs::read_dir("sample_data_dir")?;
//        for path in paths {
//            let file_name =  path.unwrap().file_name().into_string().unwrap();
//            if file_name.starts_with("thread_") {
//                //TODO
//            }
//        }

        println!("load sample dir is done: {}", sample_data_dir);
        Ok(())
    }

    //按周期滚动更换数据保存目录
    fn check_and_roll_data_dir(&mut self, sample_time: i64) -> io::Result<bool> {
        //采样文件最大时间周期
        if self.record_start_time==0 || sample_time - self.record_start_time > 900_000 {
            //create sample data dir
            let now = Local::now();
            let now_time = now.format("%Y%m%dT%H%M%S").to_string();
            let sample_data_dir = format!("{}/{}-{}", FLARE_SAMPLES_DIR, self.agent_addr.replace(":","_"), now_time);
            std::fs::create_dir_all(sample_data_dir.clone())?;
            println!("save sample data to dir: {}", sample_data_dir);

            //method info idx file
            let method_idx_path = format!("{}/method_info", sample_data_dir);
            if self.sample_method_idx_file.is_some() {
                //copy old method info file to new dir
                let old_method_file = format!("{}/method_info.fidx", self.sample_data_dir);
                let new_method_file = format!("{}/method_info.fidx", sample_data_dir);
                std::fs::copy(old_method_file, new_method_file)?;

                let old_method_file = format!("{}/method_info.fdata", self.sample_data_dir);
                let new_method_file = format!("{}/method_info.fdata", sample_data_dir);
                std::fs::copy(old_method_file, new_method_file)?;
            }
            let mut method_idx_file = TupleIndexedFile::new_writer(&method_idx_path, ValueType::INT64)?;

            self.record_start_time = sample_time;
            self.sample_data_dir = sample_data_dir;
            self.sample_method_idx_file = Some(method_idx_file);
            self.sample_cpu_ts_map.clear();
            self.sample_stacktrace_map.clear();
            self.sample_cpu_ts_cache.clear();
            //reset sample count
            for thread in self.threads.values_mut() {
                thread.sample_count = 0;
            }
            Ok(true)
        }else {
            Ok(false)
        }
    }

    fn save_summary_info(&mut self) -> io::Result<()> {
        if self.readonly {
            return Ok(());
        }

        let now = Local::now().timestamp_millis();
        if now - self.last_save_time < 1000 {
            //ignore write file frequently
            return Ok(());
        }

        let mut info = SummaryInfo {
            sample_info: self.get_sample_info(),
            threads: vec![]
        };
        for thread in self.threads.values() {
            info.threads.push(thread.clone());
        }

        let path = format!("{}/summary_info.json", self.sample_data_dir);
        match file_utils::open_file(&path, true) {
            Ok(mut file) => {
                let json = serde_json::to_string_pretty(&info).unwrap();
                file.write_all(json.as_bytes());
                file.set_len(json.as_bytes().len() as u64);
                self.last_save_time = Local::now().timestamp_millis();
                Ok(())
            }
            Err(e) => {
                println!("save summary info failed: {}", e);
                Err(e)
            }
        }
    }

    pub fn connect_agent(&mut self) -> io::Result<TcpStream> {
        self.sample_type = "attach".to_string();
        match TcpStream::connect(&self.agent_addr) {
            Ok(mut stream) => {
                println!("Successfully connected to flare agent at: {:?}", self.agent_addr);
                self.connected = true;
                Ok(stream)
            }
            Err(e) => {
                println!("Failed to connect to flare agent: {:?}, error: {:?}", self.agent_addr, e);
                Err(e)
            }
        }
    }

    pub fn subscribe_events(&mut self) -> Result<bool, Error> {
        let mut stream = self.connect_agent()?;
        let cmdValue = resp::Value::Array(vec![Value::String("subscribe-events".to_string())]);
        let cmd = cmdValue.encode();
        let size = stream.write(cmd.as_slice()).unwrap();
        println!("start subscribe events, awaiting reply: {}", cmdValue.to_encoded_string()?);

        if let Some(this_ref) = &self.this_ref {
            self.agent_stream = Some(stream.try_clone()?);
            let this = this_ref.clone();
            std::thread::spawn(move ||{
                let mut decoder = resp::Decoder::with_buf_bulk(BufReader::new(stream));
                while match decoder.decode() {
                    Ok(data) => {
                        this.lock().unwrap().on_sample_data(data)
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                        false
                    }
                }{}
                println!("subscribe events is stopped.");
                this.lock().unwrap().on_disconnected();
            });
        }
        Ok(true)
    }

    fn on_disconnected(&mut self) {
        self.running = false;
        self.disconnected = true;
    }

    fn on_sample_data(&mut self, sample_data: resp::Value) -> bool {
        if !self.running {
            return false;
        }
        //println!("events: \n{}", sample_data.to_string_pretty());
        if let resp::Value::Array(data_vec) = sample_data {
            if let Value::String(cmd) = &data_vec[0] {
                if cmd == "method" {
                    self.on_method_data(&data_vec);
                } else if cmd == "thread" {
                    self.on_thread_data(&data_vec);
                } else if cmd == "sample_info" {
                    self.on_sample_info_data(&data_vec);
                }
            }
        }

        self.save_summary_info();
        true
    }

    fn on_sample_info_data(&mut self, data_vec: &Vec<Value>) {
        let start_time= get_resp_property_as_int(data_vec, "start_time", 1, 0);
        let sample_interval= get_resp_property_as_int(data_vec, "sample_interval", 1, 0);
        let last_sample_time= get_resp_property_as_int(data_vec, "last_sample_time", 1, 0);
        self.sample_start_time = start_time;
        self.sample_interval = sample_interval;
        println!("on sample info: start_time:{}, sample_interval:{}", start_time, sample_interval);

        self.check_and_roll_data_dir(last_sample_time);
    }

    fn on_method_data(&mut self, data_vec: &Vec<Value>) {
        if let Some(Value::Integer(method_id)) = get_resp_property(data_vec, "id", 1) {
            if let Some(Value::String(method_name)) = get_resp_property(data_vec, "name", 1) {
                self.save_method_info(*method_id, method_name);
            }else {
                println!("parse method name failed")
            }
        }else {
            println!("parse method id failed")
        }
    }

    fn on_thread_data(&mut self, data_vec: &Vec<Value>) -> io::Result<()> {
        //let map = parse_resp_properties(data_vec, 1);
        let sample_time= get_resp_property_as_int(data_vec, "time", 1, 0);
        let thread_id= get_resp_property_as_int(data_vec, "id", 1, 0);
        let cpu_time= get_resp_property_as_int(data_vec, "cpu_time", 1, 0);
        let cpu_time_delta= get_resp_property_as_int(data_vec, "cpu_time_delta", 1, 0);
        let name= get_resp_property_as_str(data_vec, "name", 1, "");
        let state= get_resp_property_as_str(data_vec, "state", 1, "");
        let mut stacktrace = &vec![];
        if let Some(Value::Array(x)) = get_resp_property(data_vec, "stacktrace", 1) {
            stacktrace = x;
        }

        //create thread cpu ts
        let mut is_new = false;
        let thread_data = self.threads.entry(thread_id).or_insert_with(||{
            is_new = true;
            ThreadData {
                id: thread_id,
                name: name.to_string(),
                priority: 0,
                daemon: false,
                state: state.to_string(),
                cpu_time: cpu_time,
                cpu_time_delta: cpu_time_delta,
                sample_time: sample_time,
                sample_count: 0,
                stacktrace: vec![],
                duration: 0,
                self_duration: 0,
                self_cpu_time: 0
            }
        });
        thread_data.sample_time = sample_time;
        thread_data.sample_count += 1;
        thread_data.cpu_time = cpu_time;
        thread_data.cpu_time_delta = cpu_time_delta;
        thread_data.state = state.to_string();
        thread_data.name = name.to_string();

        //stacktrace
        let mut stack_frames = Vec::with_capacity(stacktrace.len());
        for frame in stacktrace {
            if let Value::Integer(method_id) = frame {
                stack_frames.push(*method_id);
            }
        }
        thread_data.stacktrace = stack_frames;
        //clone: break mut ref of self
        let thread_data = thread_data.clone();

        //prepare data dir
        self.check_and_roll_data_dir(sample_time);
        self.last_record_time = sample_time;
        if is_new {
            self.save_summary_info();
        }

        //save thread cpu time
        let sample_interval = self.sample_interval as i32;
        let sample_data_dir = &self.sample_data_dir;
        let cpu_ts = self.sample_cpu_ts_map.entry(thread_id).or_insert_with(||{
            let path = format!("{}/thread_{}_cpu_time", sample_data_dir, thread_id);
            match TimeSeriesFileWriter::new(ValueType::INT32, sample_interval , sample_time, &path) {
                Ok(ts) => Some(Box::new(ts)),
                Err(e) => {
                    println!("create thread cpu ts file failed: thread_id: {}, err: {}", thread_id, e);
                    None
                }
            }
        });
        let mut ts_steps = 0u32;
        if let Some(ts) = cpu_ts {
            if let Ok(steps) = ts.add_value(sample_time, TSValue::int32((cpu_time_delta/1000 as i64) as i32)) {
                ts_steps = steps;
            }
        }

        //save thread stack data
        let thread_stack_idx = self.sample_stacktrace_map.entry(thread_id).or_insert_with(||{
            let path = format!("{}/thread_{}_stack", sample_data_dir, thread_id);
            match TupleIndexedFile::new_writer(&path, ValueType::UINT32) {
                Ok(idx_file) => Some(idx_file),
                Err(e) => {
                    println!("create thread cpu ts file failed: thread_id: {}, err: {}", thread_id, e);
                    None
                }
            }
        });
        if let Some(idx_file) = thread_stack_idx {
            //save thread stacktrace as encoded resp::Value bytes
//            let stack_data = Value::Array(stacktrace.clone());
//            idx_file.add_value(TupleValue::uint32(ts_steps), stack_data.encode().as_slice());

            let data = serde_json::to_vec(&thread_data)?;
            idx_file.add_value(TupleValue::uint32(ts_steps), &data);
        }

        Ok(())
    }

    fn save_method_info(&mut self, method_id: i64, method_name: &String) {
        if let Some(idx) = self.sample_method_idx_file.as_mut() {
            idx.add_value(TupleValue::int64(method_id), method_name.as_bytes());
        }
    }

    pub fn get_dashboard(&mut self) -> DashboardInfo {

        let mut info = DashboardInfo {
            sample_info: self.get_sample_info(),
            threads: vec![]
        };

        //println!("{:8} {:48} {:8} {:8} {:8} {:8} {:8} {:8}", "ID", "NAME", "GROUP", "PRIORITY", "STATE", "%CPU", "TIME", "DAEMON");
        for thread in self.threads.values_mut() {
            //计算CPU占用率
            let cpu_util = 1;
//            let cpu_time = thread.cpu_time / 1000_000;
            //println!("{:<8} {:<48} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}", thread.id,  &thread.name[0..min(48, thread.name.len())], "main", thread.priority, thread.state, cpu_util, cpu_time, thread.daemon );
            if thread.sample_count <= 0 {
                continue;
            }
            info.threads.push(thread.clone())
        }

        info.threads.sort_by(|a, b| b.id.cmp(&a.id).reverse());
        info
    }

    pub fn get_sample_info(&self) -> SampleInfo {
        SampleInfo {
            sample_start_time: self.sample_start_time,
            record_start_time: self.record_start_time,
            last_record_time: self.last_record_time,
            sample_interval: self.sample_interval,
            agent_addr: self.agent_addr.clone(),
            sample_data_dir: self.sample_data_dir.clone()
        }
    }

    pub fn get_sample_type(&self) -> String {
        self.sample_type.clone()
    }

    pub fn get_thread_cpu_time(&mut self, thread_id: &i64, start_time: i64, end_time: i64, unit_time_ms: i64) -> Option<Arc<TSResult>> {
//        match self.sample_cpu_ts_map.get(thread_id) {
//            Some(ts) => {
//                match ts {
//                    Some(ts) => {
//                        Some(ts.get_range_value(start_time, end_time, unit_time_ms as i32))
//                    },
//                    None => None,
//                }
//            },
//            None => None
//        }

        let ts = self.sample_cpu_ts_map.get(thread_id).unwrap_or(&None);
        if ts.is_none() {
            return None;
        }
        //只有打开取样文件才缓存CPU统计数据
        if self.sample_type == "file" {
            let cache_key = format!("thread_cpu_ts_{}_{}_{}_{}", thread_id, unit_time_ms, start_time, end_time);
            self.sample_cpu_ts_cache.entry(cache_key).or_insert_with(||{
                if let Some(tsf) = ts {
                    Some(Arc::new(tsf.get_range_value(start_time, end_time, unit_time_ms as i32)))
                }else {
                    None
                }
            }).clone()
        } else {
            if let Some(tsf) = ts {
                Some(Arc::new(tsf.get_range_value(start_time, end_time, unit_time_ms as i32)))
            }else {
                None
            }
        }
    }

    pub fn get_collapsed_call_stacks(&mut self, thread_id: i64, start_time: i64, end_time: i64, stats_type: StatsType) -> io::Result<Vec<String>> {
        let mut start_step = 0;
        let mut end_step = 0;
        let mut sw = Stopwatch::start_new();
        sw.start();
        if let Some(ts_file) = self.sample_cpu_ts_map.get(&thread_id).unwrap_or(&None) {
            start_step = ts_file.time_to_step(start_time);
            end_step = ts_file.time_to_step(end_time);
        }else {
            return Err(new_error(ErrorKind::NotFound, "thread cpu time file not found"));
        }
        println!("thread: {}, convert time to step cost:{}, steps:{}", thread_id, sw.lap(), end_step-start_step);

        //TODO 可能单次读取的数据比较多，导致内存消耗太大
        let mut thread_data_vec = vec![];
        let mut last_sample_time = 0;
        self.sample_stacktrace_map.get_mut(&thread_id).unwrap_or(&mut None).as_mut().map(|idx_file| {
            idx_file.get_range_value(&TupleValue::uint32(start_step), &TupleValue::uint32(end_step), |bytes|{
                //parse stack data
                if let Ok(mut thread_data) = serde_json::from_slice::<ThreadData>(bytes.as_slice()) {
                    if last_sample_time != 0 {
                        thread_data.self_duration = thread_data.sample_time - last_sample_time;
                    }
                    last_sample_time = thread_data.sample_time;
                    thread_data_vec.push(thread_data);
                }
            });
        });
        println!("thread: {}, load stacktrace cost:{}, count:{}", thread_id, sw.lap(), thread_data_vec.len());

        let mut collapsed_stacks = vec![];
        let mut last_cpu_time = 0;
        for thread_data in &thread_data_vec {
            let mut collapsed_stack = String::new();
            for method in thread_data.stacktrace.iter().rev() {
                if let Some(method_info) = self.get_method_info(*method) {
                    if collapsed_stack.len() > 0 {
                        collapsed_stack += ";";
                    }
                    collapsed_stack += &method_info.full_name;
                }else {
                    collapsed_stack += ";";
                    collapsed_stack += &method.to_string();
                }
            }
            //get stats value
            let stats_value = match stats_type {
                StatsType::DURATION => thread_data.self_duration,
                StatsType::CPU_TIME => thread_data.self_cpu_time,
                StatsType::SAMPLES => 1,
            };
            collapsed_stack += " ";
            collapsed_stack += &stats_value.to_string();
            collapsed_stacks.push(collapsed_stack);
        }

        Ok(collapsed_stacks)
    }

    //获取顺序排列（时间顺序）的方法调用树
    pub fn get_sequenced_call_tree(&mut self, thread_id: i64, start_time: &mut i64, end_time: &mut i64) -> io::Result<Box<tree::TreeNode>> {
        let mut start_step = 0;
        let mut end_step = 0;
        let mut sw = Stopwatch::start_new();
        sw.start();
        let mut ts_file_begin_time = 0;
        if let Some(ts_file) = self.sample_cpu_ts_map.get(&thread_id).unwrap_or(&None) {
            ts_file_begin_time = ts_file.get_begin_time();
            start_step = ts_file.time_to_step(*start_time);
            end_step = ts_file.time_to_step(*end_time);
        } else {
            return Err(new_error(ErrorKind::NotFound, "thread cpu time file not found"));
        }
        println!("thread: {}, convert time to step cost:{}, steps:{}", thread_id, sw.lap(), end_step - start_step);
        let last_time = *end_time - ts_file_begin_time;

        //TODO 可能单次读取的数据比较多，导致内存消耗太大
        //TODO fix range
        let mut thread_data_vec = vec![];
        let mut last_thread_data: Option<ThreadData> = None;
        self.sample_stacktrace_map.get_mut(&thread_id).unwrap_or(&mut None).as_mut().map(|idx_file| {
            idx_file.get_range_value(&TupleValue::uint32(start_step), &TupleValue::uint32(end_step), |bytes| {
                //parse stack data
                if let Ok(mut thread_data) = serde_json::from_slice::<ThreadData>(bytes.as_slice()) {
                    if last_thread_data.is_some() {
                        let mut last_call = last_thread_data.take().unwrap();
                        last_call.self_duration = thread_data.sample_time - last_call.sample_time;
                        thread_data_vec.push(last_call);
                    }
                    last_thread_data = Some(thread_data);
                }
            });
        });
        //last method call
        if let Some(mut last_call) = last_thread_data {
            // how long of last method call duration?
            if last_time > last_call.sample_time {
                last_call.self_duration = last_time - last_call.sample_time;
                thread_data_vec.push(last_call);
            }else {
                //last sample time is out of range, drop it
            }
        }
        println!("thread: {}, load stacktrace cost:{}, count:{}, step: {} - {}", thread_id, sw.lap(), thread_data_vec.len(), start_step, end_step);

        thread_data_vec.first_mut().map(|thread_data|{
            *start_time = thread_data.sample_time;
        });
        thread_data_vec.last_mut().map(|thread_data|{
            *end_time = thread_data.sample_time + thread_data.duration;
        });

        //merge build
        self.build_ordinal_tree(&thread_data_vec, *start_time, *end_time)
    }

    //火焰图的顺序树
    //每一层与最后一个节点相同时进行合并，不同时append新节点
    //处理前后半个采样间隔的问题
    pub fn build_ordinal_tree(&mut self, thread_data_vec: &Vec<ThreadData>, range_start_time: i64, range_end_time: i64) -> io::Result<Box<tree::TreeNode>> {
        let mut root = Box::new(tree::TreeNode::new(0, "root"));

        for thread_data in thread_data_vec {
            root.duration += thread_data.self_duration;
            root.cpu += thread_data.self_cpu_time;
            root.calls += 1;
            let mut node = &mut root;
            let mut start_time = thread_data.sample_time - range_start_time;
            if start_time < 0 {
                start_time = 0;
            }
            for method in thread_data.stacktrace.iter().rev() {
                let tmp;
                let method_name = if let Some(method_info) = self.get_method_info(*method) {
                    &method_info.full_name
                }else {
                    tmp = method.to_string();
                    &tmp
                };
                //merge_last_child fn return bool instead of node reference for avoiding second borrow mutable node
                if node.merge_last_child(method_name, thread_data.self_duration, thread_data.self_cpu_time, 1) {
                    //merge success, next is just last child
                    node = node.last_child().unwrap();
                } else {
                    let child_depth = node.depth+1;
                    node = node.append_child(tree::TreeNode{
                        parent: None,
                        children: vec![],
                        depth: child_depth,
                        id: 0,
                        label: method_name.to_string(),
                        calls: 1,
                        cpu: thread_data.self_cpu_time,
                        duration: thread_data.self_duration,
                        start_time
                    })
                }
            }
        }
        Ok(root)
    }

    pub fn get_call_tree(&mut self, thread_ids: &[i64], start_time: i64, end_time: i64) -> io::Result<CallStackTree> {
        //TODO
        let mut stack_tree = CallStackTree::new(0, "CallStack");
        let mut sw = Stopwatch::start_new();

        for thread_id in thread_ids {
            let mut start_step = 0;
            let mut end_step = 0;
            sw.start();
            if let Some(ts_file) = self.sample_cpu_ts_map.get(thread_id).unwrap_or(&None) {
                start_step = ts_file.time_to_step(start_time);
                end_step = ts_file.time_to_step(end_time);
            }else {
                continue;
            }
            println!("thread: {}, convert time to step cost:{}, steps:{}", thread_id, sw.lap(), end_step-start_step);

            //TODO 可能单次读取的数据比较多，导致内存消耗太大
            let mut thread_data_vec = vec![];
            let mut last_sample_time = 0;
            self.sample_stacktrace_map.get_mut(thread_id).unwrap_or(&mut None).as_mut().map(|idx_file| {
                idx_file.get_range_value(&TupleValue::uint32(start_step), &TupleValue::uint32(end_step), |bytes|{
                    //parse stack data
                    if let Ok(mut thread_data) = serde_json::from_slice::<ThreadData>(bytes.as_slice()) {
                        if last_sample_time != 0 {
                            thread_data.duration = thread_data.sample_time - last_sample_time;
                        }
                        last_sample_time = thread_data.sample_time;
                        thread_data_vec.push(thread_data);
                    }
                });
            });
            println!("thread: {}, load stacktrace cost:{}, count:{}", thread_id, sw.lap(), thread_data_vec.len());

            //thread cpu_time 延时更新，暂时将增量时间平均分配到两次更新CPU时间中的方法调用上
            let mut last_divide_cpu_time = 0;
            let mut start = 0;
            let mut end = 0;
            let mut pending_thread_data_vec: Vec<&mut ThreadData> = vec![];
            for (i,thread_data) in thread_data_vec.iter_mut().enumerate() {
                if last_divide_cpu_time == 0 {
                    last_divide_cpu_time = thread_data.cpu_time;
                }
                if thread_data.cpu_time != last_divide_cpu_time {
                    end = i;
                    let curr_cpu_time = thread_data.cpu_time;
                    self.divide_cpu_time(pending_thread_data_vec, last_divide_cpu_time, curr_cpu_time);
                    pending_thread_data_vec = vec![];
                    last_divide_cpu_time = curr_cpu_time;
                    start = end;
                }
                pending_thread_data_vec.push(thread_data);
            }

            for thread_data in &thread_data_vec {
                self.add_stack_trace(&mut stack_tree, thread_data);
            }
            println!("thread: {}, build tree cost:{}", thread_id, sw.lap());

//            if let Some(method_idx) = self.sample_method_idx_file.as_mut() {
//                for method in stack_frames {
//                    let bytes = method_idx.get_value(&TupleValue::int64(method))?;
//                    let method_name = std::str::from_utf8(bytes.as_slice())?;
//                }
//            }

        }
        println!("total threads: {}, total cost:{}", thread_ids.len(), sw.elapsed_ms());

        Ok(stack_tree)
    }

    fn divide_cpu_time(&mut self, mut thread_data_batch: Vec<&mut ThreadData>, last_cpu_time: i64, curr_cpu_time: i64){
        let cpu_time_per_trace = (curr_cpu_time - last_cpu_time) / thread_data_batch.len() as i64;
        let mut thread_cpu_time = last_cpu_time;
        let mut i=0;
        let len = thread_data_batch.len();
        for thread_data in thread_data_batch.as_mut_slice() {
            if i < len {
                thread_cpu_time += cpu_time_per_trace;
            }else {
                thread_cpu_time = curr_cpu_time;
            }
            i += 1;
            (*thread_data).cpu_time = thread_cpu_time;
            //thread_data.cpu_time_delta =
        }
    }

    fn add_stack_trace(&mut self, call_tree: &mut CallStackTree, thread_data: &ThreadData) {

        call_tree.reset_top_call_stack_node();
        let (delta_duration, delta_cpu_time) = call_tree.start_call_stack(thread_data.sample_time, thread_data.cpu_time);

        //save nodes in temp vec, process it after build call tree, avoid second borrow muttable *self
        let mut naming_nodes: Vec<(NodeId, JavaMethod)> = vec![];

        //reverse call
        for method_id in thread_data.stacktrace.iter().rev() {
            if !call_tree.begin_call(method_id, delta_duration, delta_cpu_time) {
                naming_nodes.push((call_tree.get_top_node().data.node_id, method_id.clone()));
            }
        }

        //call_tree.end_last_call(cpu_time);
        //println!("add call stack: {} cpu_time:{}", thread_info.name, cpu_time);

        //get method call_name of node
        for (node_id, method_id) in naming_nodes {
            if let Some(method_info) = self.get_method_info(method_id) {
                call_tree.get_mut_node(&node_id).data.name = method_info.full_name.clone();

            }
        }
    }

    fn get_method_info(&mut self, method: JavaMethod) -> &Option<MethodInfo> {
        let method_idx_file = self.sample_method_idx_file.as_mut();
        self.method_cache.entry(method).or_insert_with(|| {
            if let Some(method_idx) = method_idx_file {
                if let Ok(bytes) = method_idx.get_value(&TupleValue::int64(method)){
                    let mut method_name = std::str::from_utf8(bytes.as_slice()).unwrap_or("").to_string();
                    if method_name == "" {
                        method_name = method.to_string();
                    }
                    return Some(MethodInfo {
                        method_id: method,
                        full_name: method_name.to_string(),
                        hits_count: 0
                    });
                }
            }
            return None;
        })
    }

//    pub fn write_all_call_trees(&self, writer: &mut std::io::Write, compact: bool) {
//        for (thread_id, call_tree) in self.tree_arena.get_all_call_trees() {
//            let tree_name = &call_tree.get_root_node().data.name;
//            writer.write_fmt(format_args!("Thread: {}, {}, {}\n", &call_tree.thread_id, tree_name, call_tree.total_duration as f64/1000_000.0));
//
//            writer.write_all(call_tree.format_call_tree(compact).as_bytes());
//            writer.write_all("\n".as_bytes());
//        }
//    }
//
//    pub fn format_stack_traces(&mut self, jvm_env: &Box<Environment>, stack_traces: &Vec<JavaStackTrace>) -> String {
//        let mut result  = String::new();
//        for (i, stack_info) in stack_traces.iter().enumerate() {
//            result.push_str(&format!("\nstack_info: {}, thread: {:?}, state: {:?}\n", (i+1), stack_info.thread, stack_info.state));
//
//            let mut cpu_time = -1f64;
//            match jvm_env.get_thread_cpu_time(&stack_info.thread) {
//                Ok(t) => { cpu_time = t as f64 / 1000_000.0 },
//                Err(err) => {
//                    result.push_str(&format!("get_thread_cpu_time error: {:?}\n", err))
//                }
//            }
//
//            if let Ok(thread_info) = jvm_env.get_thread_info(&stack_info.thread) {
//                result.push_str(&format!("Thread {}: (id = {}, priority = {}, daemon = {}, state = {:?}, cpu_time = {}) \n",
//                                         thread_info.name, thread_info.thread_id,  thread_info.priority, thread_info.is_daemon, stack_info.state, cpu_time ));
//            } else {
//                result.push_str(&format!("Thread UNKNOWN [{:?}]: (cpu_time = {}) \n", stack_info.thread, cpu_time));
//            }
//
//            for stack_frame in &stack_info.frame_buffer {
//                let method_info = self.get_method_info(jvm_env, stack_frame.method);
//                result.push_str(&format!("{}.{}()\n", &method_info.class.name, &method_info.method.name));
//            }
//        }
//        result
//    }
//

}


impl Drop for SampleCollector {
    fn drop(&mut self) {
        println!("dropping sample collector: {} ..", self.sample_data_dir);
        self.save_summary_info();
        self.close();
    }
}