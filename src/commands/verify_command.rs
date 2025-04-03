use serenity::builder::{CreateButton, CreateEmbed, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::str::FromStr;

use crate::Args;
use crate::config::Config;
use crate::user::User;
use crate::verification::{PendingVerifications, VerificationInfo};

const NOT_CONFIGURED: &'static str =
    "The bot is not yet configured, an admin needs to use the /config command";

async fn get_user_data(account: &str) -> Option<User> {
    if account.starts_with("https://osu.ppy.sh/users/") || account.starts_with("osu.ppy.sh/users/")
    {
        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "users" {
                let user_id = parts.next()?;
            }
        }
        return None;
    }

    None
}

fn create_profile_embed(user: &User) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("Mania profile for {}", user.username))
        .image(user.avatar_url.clone())
        .description(format!(
            "**- Country:** {country}\n
                        **- Rank:** Global: #{rank} | Country: #{country_rank}\n
                        [{link}]
                        ",
            country = user.country.trim(),
            rank = user.ranks.global.unwrap_or(0),
            country_rank = user.ranks.country.unwrap_or(0),
            link = user.link,
        ))
        .color(0xCE7AFF)
}

pub async fn execute(
    ctx: &Context,
    channel_id: &ChannelId,
    member: Member,
    args: Args,
) -> Result<(), String> {
    let user = get_user_data(&args.arg(0).ok_or("No argument supplied")?).await;

    let mut data = ctx.data.write().await;

    let config = Config::load().ok_or("Could not load config")?;

    if let Some(verification_channel) = config.channels.verification_channel {
        if *channel_id != verification_channel {
            return Ok(());
        }
    } else {
        return Err(NOT_CONFIGURED.to_string());
    }

    let admin_channel = config
        .channels
        .admin_channel
        .ok_or(NOT_CONFIGURED.to_string())?;

    if let Some(user) = user {
        let verifications = data
            .get_mut::<PendingVerifications>()
            .expect("No verification hashmap found");
        let id = verifications.use_current_id();

        let country = country_from_code(&user.country);

        let embed = create_profile_embed(&user);

        let status_embed = CreateEmbed::new()
            .title(format!(
                "Verification Request for {}",
                member.user.display_name()
            ))
            .description("**Current status:** 🟡 Pending");

        let verify_button = CreateButton::new("verify ".to_string() + &id.to_string())
            .label("Click here to verify");

        let deny_button =
            CreateButton::new("deny ".to_string() + &id.to_string()).label("Click here to decline");

        let message = CreateMessage::new()
            .embed(embed)
            .button(verify_button)
            .button(deny_button);
        let message = admin_channel
            .send_message(&ctx.http, message)
            .await
            .expect("Failed to send verification message");

        let status_message = CreateMessage::new().embed(status_embed);
        let status_message = channel_id
            .send_message(&ctx.http, status_message)
            .await
            .expect("Failed to send status message");

        let verification_info = VerificationInfo {
            discord_user: member,
            osu_username: user.username.to_string(),
            country: country.trim().to_string(),
            status_message,
            verification_message: message,
        };

        verifications.insert(id, verification_info);
    };
    Ok(())
}

fn country_from_code(code: &str) -> &'static str {
    celes::Country::from_str(code).unwrap().long_name
}
