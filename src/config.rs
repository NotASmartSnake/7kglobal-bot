use serenity::model::prelude::*;
use serenity::prelude::*;

use serde::{Deserialize, Serialize};

use std::fs::{self, File};
use std::io::Write;

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub admin_channel: Option<ChannelId>,
    pub verification_channel: Option<ChannelId>,
}

impl TypeMapKey for Config {
    type Value = Config;
}

impl Config {
    pub fn save_to_file(&self, file: &str) -> Result<(), String> {
        let config = serde_json::to_string(self).unwrap();
        let mut file = File::create(file).unwrap();
        if let Err(e) = file.write_all(config.as_bytes()) {
            return Err(format!("Could not write to file: {e}"));
        }
        Ok(())
    }

    pub fn load_from_file(file: &str) -> Option<Self> {
        let contents = fs::read_to_string(file).ok()?;
        let config: Config = serde_json::from_str(&contents).ok()?;
        Some(config)
    }
}
