use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Default)]
pub struct Config {
    pub admin_channel: Option<ChannelId>,
    pub verification_channel: Option<ChannelId>,
}

impl TypeMapKey for Config {
    type Value = Config;
}
