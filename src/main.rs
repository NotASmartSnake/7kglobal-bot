use serenity::model::prelude::*;
use serenity::prelude::*;

use std::env;
use std::str::FromStr;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD-TOKEN")?;
    let mut client = Client::builder(token, GatewayIntents::default())
        .event_handler(sevenkey_global_bot::Handler)
        .await?;

    client.start().await?;
    Ok(())
}
