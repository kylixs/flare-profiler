extern crate flare_client;

use flare_client::sampler_client::*;
use flare_client::*;
use std::sync::{Mutex, Arc};

fn main() {

//    match SamplerClient::new("localhost:3333") {
//        Ok(mut client) => {
//            client.subscribe_events();
//        }
//        Err(e) => {
//            println!("start sampler client failed: {:?}", e);
//        }
//    }

    let mut profiler = Profiler::new();
//    profiler.lock().unwrap().connect_agent("localhost:3333");

    //start websocket server
    profiler.lock().unwrap().startup();


//    let timer = timer::Timer::new();
//    let profiler_ref = profiler.clone();
//    let guard = {
//        timer.schedule_repeating(chrono::Duration::milliseconds(3000), move || {
//            println!("=======================================================");
//            profiler_ref.lock().unwrap().get_dashboard();
//        })
//    };

    //wait for closing
    loop {
        if !profiler.lock().unwrap().is_running() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

//    drop(guard);
}