extern crate flareclient;

use flareclient::sampler_client::*;


fn main() {
    match SamplerClient::new("localhost:3333") {
        Ok(mut client) => {
            client.subscribe_events();
        }
        Err(e) => {
            println!("start sampler client failed: {:?}", e);
        }
    }
}