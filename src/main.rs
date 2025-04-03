use serenity::model::prelude::*;
use serenity::prelude::*;

use sevenkey_global_bot::GuildKey;
use sevenkey_global_bot::verification::PendingVerifications;

use std::env;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")?;
    let mut client = Client::builder(
        token,
        GatewayIntents::default() | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(sevenkey_global_bot::Handler)
    .await?;

    let client_id = env::var("OSU_API_ID")?.parse::<u64>()?;
    let client_secret = env::var("OSU_API_SECRET")?;
    let guild_id = GuildId::new(env::var("GUILD_ID")?.parse::<u64>()?);

    {
        let mut data = client.data.write().await;
        data.insert::<PendingVerifications>(PendingVerifications::default());
        data.insert::<GuildKey>(guild_id);
    }

    client.start().await?;
    Ok(())
}
