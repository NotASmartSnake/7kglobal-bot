use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use rusqlite::Connection;

use crate::Args;

fn get_game_counts_buf() -> Result<String, String> {
    let conn = Connection::open("users.db").map_err(|_| "Database failure")?;

    let mut stmt = conn
        .prepare(
            "SELECT COUNT(discord_id), game
            FROM users 
            GROUP BY game
            ORDER BY COUNT(discord_id) DESC;",
        )
        .map_err(|_| "Database failure")?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0), row.get::<_, String>(1))))
        .map_err(|_| "Database failure")?;

    let mut buf = String::new();

    for row in rows.take(10) {
        if let Ok(row) = row {
            buf += format!(
                "**{}**: {}\n",
                row.1.unwrap_or("Undefined Game".to_string()),
                row.0.unwrap_or(0)
            )
            .as_str();
        }
    }

    Ok(buf)
}
fn get_country_counts_buf() -> Result<String, String> {
    let conn = Connection::open("users.db").map_err(|_| "Database failure")?;

    let mut stmt = conn
        .prepare(
            "SELECT COUNT(discord_id), country 
            FROM users 
            GROUP BY country 
            ORDER BY COUNT(discord_id) DESC;",
        )
        .map_err(|_| "Database failure")?;

    let rows = stmt
        .query_map([], |row| Ok((row.get(0), row.get::<_, String>(1))))
        .map_err(|_| "Database failure")?;

    let mut buf = String::new();

    for row in rows.take(10) {
        if let Ok(row) = row {
            buf += format!(
                "**{}**: {}\n",
                row.1.unwrap_or("Undefined Country".to_string()),
                row.0.unwrap_or(0)
            )
            .as_str();
        }
    }

    Ok(buf)
}

async fn list_by_country(ctx: &Context, channel_id: &ChannelId) -> Result<(), String> {
    let countries = get_country_counts_buf()?;

    let embed = CreateEmbed::new()
        .title("Members by country:")
        .description(countries);

    channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(embed))
        .await
        .expect("Message failed to send");

    Ok(())
}

async fn list_by_game(ctx: &Context, channel_id: &ChannelId) -> Result<(), String> {
    let games = get_game_counts_buf()?;

    let embed = CreateEmbed::new()
        .title("Members by game:")
        .description(games);

    channel_id
        .send_message(&ctx.http, CreateMessage::new().embed(embed))
        .await
        .expect("Message failed to send");

    Ok(())
}

pub async fn execute(
    ctx: &Context,
    channel_id: &ChannelId,
    _member: &Member,
    args: Args,
) -> Result<(), String> {
    match args.arg(0).ok_or("Expected an argument".to_string())? {
        "country" => list_by_country(ctx, channel_id).await?,
        "game" => list_by_game(ctx, channel_id).await?,
        _ => return Err("Invalid argument".to_string()),
    }

    Ok(())
}
