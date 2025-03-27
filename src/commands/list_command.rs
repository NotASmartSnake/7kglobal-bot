use crate::{Args, GuildKey};
use serenity::model::prelude::*;
use serenity::prelude::*;

fn is_country_role(role_name: &str) -> bool {
    match role_name {
        "Member" => false,
        "BN" => false,
        "Bot" => false,
        _ => true,
    }
}

async fn list_by_country(ctx: &Context, channel_id: &ChannelId, args: Args) -> Result<(), String> {
    let data = ctx.data.read().await;
    let guild_id = data.get::<GuildKey>().expect("There is no guild id");
    let roles = guild_id
        .roles(&ctx.http)
        .await
        .map_err(|_| "Could not get roles")?;

    let roles = roles
        .values()
        .filter(|role| is_country_role(&role.name))
        .collect::<Vec<Role>>();

    Ok(())
}

pub async fn execute(ctx: &Context, channel_id: &ChannelId, args: Args) -> Result<(), String> {
    match args.arg(0).ok_or("Expected an argument".to_string())? {
        "Country" => list_by_country(ctx, channel_id, args).await?,
        _ => return Err(format!("Invalid argument: {}", args.arg(0).unwrap())),
    }

    Ok(())
}
