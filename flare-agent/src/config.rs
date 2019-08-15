extern crate toml;

use std::fs::File;
use std::io::{ Read };
use std::path::Path;

#[derive(Deserialize,Debug)]
pub struct Config {
    pub agent_name: String,
    pub entry_points: Vec<String>,
    pub active_classes: Vec<String>,
    pub trace_enable: bool
}

impl Config {

    pub fn read_config() -> Option<Config> {
        let default_config: String = String::from("agent.conf");

        Config::read_from_file(default_config)
    }

    pub fn read_from_file<T: AsRef<Path>>(file_name: T) -> Option<Config> {
        match File::open(file_name) {
            Ok(mut file) => {
                let mut contents = String::new();
                let _ = file.read_to_string(&mut contents);

                let config: Config = toml::from_str(contents.as_str()).unwrap();

                Some(config)
            },
            _ => None
        }
    }
}

impl Default for Config {

    fn default() -> Self {
        Config {
            agent_name: String::from("default"),
            entry_points: vec![],
            active_classes: vec![],
            trace_enable: true
        }
    }
}
