use std::collections::HashMap;

///
/// Represents the possible program options configured from command-line arguments.
///
#[derive(Debug)]
pub struct Options {
    pub agent_id: String,
    pub custom_args: HashMap<String, String>,
    pub config_location: Option<String>
}

impl Options {

    /// Turn a string of command-line arguments into an `Options` instance
    pub fn parse(opt_args: String) -> Options {
        let mut options = Options::default();

        if opt_args.len() > 0 {
            for arg in opt_args.split(",") {
                match arg.find('=') {
                    Some(position) => {
                        let (key, value) = arg.split_at(position);
                        Options::parse_key_value(&mut options, key, &value[1..value.len()]);
                    },
                    None => { Options::parse_directive(&mut options, arg); }
                }
            }
        }

        options
    }

    fn parse_key_value(options: &mut Options, key: &str, value: &str) {
        println!("Parsing key: {} -> {}", key, value);
        match key {
            "agentid" => { options.agent_id = value.to_string(); },
            "config" => { options.config_location = Some(value.to_string()); },
            _ => { options.custom_args.insert(key.to_string(), value.to_string()); }
        }
    }

    fn parse_directive(options: &mut Options, directive: &str) {
        match directive {
            _ => options.custom_args.insert(directive.to_string(), "".to_string())
        };
    }

    /// Return the default configuration options
    pub fn default() -> Options {
        Options {
            agent_id: "jvmti".to_string(),
            custom_args: HashMap::new(),
            config_location: None
        }
    }
}
