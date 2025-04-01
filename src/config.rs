use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

#[derive(Default, Serialize, Deserialize)]
pub struct Channels {
    pub admin_channel: Option<ChannelId>,
    pub verification_channel: Option<ChannelId>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub channels: Channels,
    pub emoji_exceptions: HashMap<String, String>,
}

impl TypeMapKey for Config {
    type Value = Config;
}

impl Config {
    pub fn save(&self) -> Result<(), String> {
        let config = serde_json::to_string(self).unwrap();
        let mut file = File::create("config.json").unwrap();
        if let Err(e) = file.write_all(config.as_bytes()) {
            return Err(format!("Could not write to file: {e}"));
        }
        Ok(())
    }

    pub fn load() -> Option<Self> {
        let contents = fs::read_to_string("config.json").ok()?;
        let config: Config = serde_json::from_str(&contents).ok()?;
        Some(config)
    }
}
