use crate::{Args, GuildKey};
use serenity::builder::{CreateEmbed, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;

type Roles = HashMap<String, u32>;

fn is_country_role(role_name: &str) -> bool {
    match role_name.to_lowercase().as_str() {
        "member" => false,
        "bn" => false,
        "bot" => false,
        "7k global bot" => false,
        "7k global testing bot" => false,
        "carl-bot" => false,
        _ => true,
    }
}

async fn cache_members(ctx: &Context, members: &[Member], roles: &mut Roles) {
    for member in members.iter() {
        for role in member.roles(&ctx.cache).unwrap().iter() {
            if let Some(count) = roles.get_mut(&role.name) {
                *count += 1;
            }
        }
    }

    let json = serde_json::to_string(roles).unwrap();
    let mut file = File::create("member_role_cache.json").unwrap();

    file.write_all(json.as_bytes()).unwrap();
}

fn get_roles_from_cache() -> Option<Roles> {
    let contents = fs::read_to_string("member_role_cache.json").ok()?;
    serde_json::from_str(&contents).ok()?
}

async fn list_by_country(
    ctx: &Context,
    channel_id: &ChannelId,
    member: &Member,
    args: Args,
) -> Result<(), String> {
    let data = ctx.data.read().await;
    let guild_id = data.get::<GuildKey>().expect("There is no guild id");
    let roles = guild_id
        .roles(&ctx.http)
        .await
        .map_err(|_| "Could not get roles")?;

    let mut country_roles = roles
        .values()
        .filter(|role| is_country_role(&role.name))
        .map(|role| (role.name.clone(), 0))
        .collect::<Roles>();

    country_roles = if let Some("recache") = args.arg(1) {
        let guild_channel = match channel_id
            .to_channel(&ctx.http)
            .await
            .map_err(|_| "Could not get channel from channel id")?
        {
            Channel::Guild(guild_channel) => guild_channel,
            Channel::Private(_) => return Err("This bot cannot be used in dms".to_string()),
            _ => return Err("Unexpected Channel Type".to_string()),
        };

        let user_permissions = guild_id
            .to_partial_guild(&ctx.http)
            .await
            .map_err(|_| "Could not get guild from id")?
            .user_permissions_in(&guild_channel, member);

        if user_permissions.contains(Permissions::ADMINISTRATOR) {
            cache_members(
                &ctx,
                &guild_id
                    .members(&ctx.http, None, None)
                    .await
                    .map_err(|e| format!("Could not get members: {e}"))?,
                &mut country_roles,
            )
            .await;
        } else {
            return Err("Only admins can use the 'recache' argument".to_string());
        }

        country_roles
    } else {
        get_roles_from_cache().ok_or(
            "Role cache is missing, get an admin to run the command with the 'recache' argument",
        )?
    };

    let mut country_roles = Vec::from_iter(country_roles);
    country_roles.sort_by(|(_, value), (_, value2)| value.cmp(value2));

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
    member: &Member,
    args: Args,
) -> Result<(), String> {
    match args.arg(0).ok_or("Expected an argument".to_string())? {
        "country" => list_by_country(ctx, channel_id, member, args).await?,
        _ => return Err(format!("Invalid argument: {}", args.arg(0).unwrap())),
    }

    Ok(())
}
