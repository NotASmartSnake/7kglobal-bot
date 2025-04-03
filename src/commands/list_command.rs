use crate::{Args, GuildKey};
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;

use std::collections::HashMap;

type Roles = HashMap<String, u32>;

fn is_country_role(role_name: &str, config: &Config) -> bool {
    if let Some(_) = config.non_country_roles.get(role_name) {
        return false;
    }
    true
}

fn count_country_members(ctx: &Context, members: &[Member], roles: &mut Roles) {
    for member in members.iter() {
        for role in member.roles(&ctx.cache).unwrap().iter() {
            if let Some(count) = roles.get_mut(&role.name) {
                *count += 1;
            }
        }
    }
}

async fn list_by_country(ctx: &Context, channel_id: &ChannelId) -> Result<(), String> {
    let data = ctx.data.read().await;
    let guild_id = data.get::<GuildKey>().expect("There is no guild id");
    let roles = guild_id
        .roles(&ctx.http)
        .await
        .map_err(|_| "Could not get roles")?;

    let config = Config::load().unwrap_or_default();

    let mut country_roles = roles
        .values()
        .filter(|role| is_country_role(&role.name, &config))
        .map(|role| (role.name.clone(), 0))
        .collect::<Roles>();

    count_country_members(
        &ctx,
        &guild_id
            .members(&ctx.http, None, None)
            .await
            .map_err(|e| format!("Could not get members: {e}"))?,
        &mut country_roles,
    );

    let mut country_roles = Vec::from_iter(country_roles);
    country_roles.sort_by(|(_, value), (_, value2)| value.cmp(value2).reverse());

    if country_roles.len() >= 10 {
        country_roles = country_roles[0..=9].to_vec()
    }

    let mut buf = String::new();
    for (country, member_count) in country_roles.iter() {
        buf += format!("**{}**: {}\n", country, member_count).as_str();
    }

    let embed = CreateEmbed::new()
        .title("Members by country:")
        .description(buf);

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
        _ => return Err("Invalid argument".to_string()),
    }

    Ok(())
}
