use rosu_v2::prelude::*;
use serenity::builder::{CreateButton, CreateEmbed, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;
use crate::verification::{PendingVerifications, VerificationInfo};
use crate::{Args, OsuKey};

const NOT_CONFIGURED: &'static str =
    "The bot is not yet configured, an admin needs to use the /config command";

async fn get_user_data(ctx: &Context, account: &str) -> Option<UserExtended> {
    let data = ctx.data.read().await;
    let osu: &Osu = data.get::<OsuKey>().expect("API client not found");

    if account.starts_with("https://osu.ppy.sh/users/") || account.starts_with("osu.ppy.sh/users/")
    {
        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "users" {
                let user_id = parts.next()?;
                return osu
                    .user(rosu_v2::request::UserId::Id(user_id.parse::<u32>().ok()?))
                    .mode(3.into())
                    .await
                    .ok();
            }
        }
        return None;
    }

    if let Ok(user) = osu.user(account).mode(3.into()).await {
        return Some(user);
    }

    None
}

fn create_profile_embed(user: &UserExtended, statistics: &UserStatistics) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("Mania profile for {}", user.username))
        .image(user.avatar_url.clone())
        .description(format!(
            "**- Country:** {country}\n
                        **- Rank:** Global: #{rank} | Country: #{country_rank}\n
                        [https://osu.ppy.sh/users/{id}]
                        ",
            country = user.country.trim(),
            rank = statistics.global_rank.unwrap_or(0),
            country_rank = statistics.country_rank.unwrap_or(0),
            id = user.user_id,
        ))
        .color(0xCE7AFF)
}

pub async fn execute(
    ctx: &Context,
    channel_id: ChannelId,
    member: Member,
    args: Args,
) -> Result<(), String> {
    let user = get_user_data(&ctx, &args.arg(0).ok_or("No argument supplied")?).await;

    let mut data = ctx.data.write().await;

    let config = Config::load().ok_or("Could not load config")?;

    if let Some(verification_channel) = config.channels.verification_channel {
        if channel_id != verification_channel {
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
        let statistics = user.statistics.clone().expect("User has no statistics");
        let verifications = data
            .get_mut::<PendingVerifications>()
            .expect("No verification hashmap found");
        let id = verifications.use_current_id();

        let embed = create_profile_embed(&user, &statistics);

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
            country: user.country.trim().to_string(),
            status_message,
            verification_message: message,
        };

        verifications.insert(id, verification_info);
    };
    Ok(())
}
