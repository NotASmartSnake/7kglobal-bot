use serenity::prelude::TypeMapKey;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use std::io::Write;


#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Config {
    exceptions: HashMap<String, String>,
}

impl Config {
    fn new() -> Self {
        if let Ok(contents) = fs::read_to_string("config.json")
    }
}

impl TypeMapKey for Config {
    type Value = Config;
}

pub fn get_emoji_exceptions() -> Config {
    if let Ok(exception_json) = fs::read_to_string("emoji_exceptions.json") {
        let exceptions: Config = serde_json::from_str(&exception_json).unwrap();
        return exceptions;
    } else {
        let exceptions = Config::default();
        let exceptions_json = serde_json::to_string(&exceptions).unwrap();
        let mut file = File::create("emoji_exceptions.json").unwrap();
        file.write_all(exceptions_json.as_bytes()).unwrap();
        return exceptions;
    }
}
