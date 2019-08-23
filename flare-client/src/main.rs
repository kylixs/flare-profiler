extern crate flareclient;

use flareclient::sampler_client::*;


fn main() {
    let mut client = SamplerClient::new("localhost:3333");
    client.subscribe_events();
}