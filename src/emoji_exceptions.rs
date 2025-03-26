use serenity::prelude::TypeMapKey;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::fs::File;

use std::io::Write;

use std::ops::Deref;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct EmojiExceptions {
    exceptions: HashMap<String, String>,
}

impl Deref for EmojiExceptions {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.exceptions
    }
}

impl TypeMapKey for EmojiExceptions {
    type Value = EmojiExceptions;
}

pub fn get_emoji_exceptions() -> EmojiExceptions {
    if let Ok(exception_json) = fs::read_to_string("emoji_exceptions.json") {
        let exceptions: EmojiExceptions = serde_json::from_str(&exception_json).unwrap();
        return exceptions;
    } else {
        let exceptions = EmojiExceptions::default();
        let exceptions_json = serde_json::to_string(&exceptions).unwrap();
        let mut file = File::create("emoji_exceptions.json").unwrap();
        file.write_all(exceptions_json.as_bytes()).unwrap();
        return exceptions;
    }
}
