use rosu_v2::prelude::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use sevenkey_global_bot::verification::PendingVerifications;
use sevenkey_global_bot::{GuildKey, OsuKey};

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
    let guild_id = GuildId::new(env::var("GUILD-ID")?.parse::<u64>()?);

    let osu = Osu::new(client_id, client_secret).await?;

    {
        let mut data = client.data.write().await;
        data.insert::<PendingVerifications>(PendingVerifications::default());
        data.insert::<OsuKey>(osu);
        data.insert::<GuildKey>(guild_id);
    }

    client.start().await?;
    Ok(())
}
