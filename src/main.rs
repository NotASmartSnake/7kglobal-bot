use serenity::model::prelude::*;
use serenity::prelude::*;

use sevenkey_global_bot::GuildKey;
use sevenkey_global_bot::game_api::Osu;
use sevenkey_global_bot::verification::PendingVerifications;

use std::env;

use rusqlite::Connection;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")?;
    let mut client = Client::builder(
        token,
        GatewayIntents::default() | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(sevenkey_global_bot::Handler)
    .await?;

    let client_id = env::var("OSU_API_ID")?;
    let client_secret = env::var("OSU_API_SECRET")?;
    let guild_id = GuildId::new(env::var("GUILD_ID")?.parse::<u64>()?);

    let req_client = reqwest::Client::new();
    let osu = Osu::build(req_client, &client_id, &client_secret)
        .await
        .ok_or("Could not build osu client")?;

    {
        let mut data = client.data.write().await;
        data.insert::<PendingVerifications>(PendingVerifications::default());
        data.insert::<GuildKey>(guild_id);
        data.insert::<Osu>(osu);
    }

    {
        let conn = Connection::open("users.db")?;
        conn.execute(
            "create table if not exists users (
                 discord_id integer primary key,
                 game text not null,
                 player_id integer not null,
                 username text not null,
                 country text not null
             )",
            (),
        )?;
    }

    client.start().await?;
    Ok(())
}
