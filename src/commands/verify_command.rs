use super::TextCommand;

use rosu_v2::prelude::*;
use serenity::builder::{CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::OsuKey;
use crate::config::Config;
use crate::verification::{PendingVerifications, VerificationInfo};

const NOT_CONFIGURED: &'static str =
    "The bot is not yet configured, an admin needs to use the /config command";

pub struct VerifyCommand {
    osu_username: String,
}

impl VerifyCommand {
    pub fn new(osu_username: &str) -> Self {
        Self {
            osu_username: osu_username.to_string(),
        }
    }
}

async fn get_user_data(ctx: &Context, osu_username: &str) -> Option<UserExtended> {
    let data = ctx.data.read().await;
    let osu: &Osu = data.get::<OsuKey>().expect("API client not found");

    if osu_username.starts_with("https://osu.ppy.sh/users/") {
        if let Ok(user_id) = osu_username.split("/").last().unwrap().parse::<u32>() {
            return osu
                .user(rosu_v2::request::UserId::Id(user_id))
                .mode(3.into())
                .await
                .ok();
        }

        return None;
    }

    if let Ok(user) = osu.user(osu_username).mode(3.into()).await {
        return Some(user);
    }

    None
}

#[serenity::async_trait]
impl TextCommand for VerifyCommand {
    async fn execute_text(
        &self,
        ctx: &Context,
        channel_id: ChannelId,
        member: Member,
    ) -> Result<(), String> {
        let user = get_user_data(&ctx, &self.osu_username).await;

        let mut data = ctx.data.write().await;

        if let Some(verification_channel) = data
            .get::<Config>()
            .ok_or("Could not get config data")?
            .verification_channel
        {
            if channel_id != verification_channel {
                return Ok(());
            }
        } else {
            return Err(NOT_CONFIGURED.to_string());
        }

        let admin_channel = data
            .get::<Config>()
            .ok_or("Could not get config data".to_string())?
            .admin_channel
            .ok_or(NOT_CONFIGURED.to_string())?;

        if let Some(user) = user {
            let statistics = user.statistics.expect("User no statistics");

            let verifications = data
                .get_mut::<PendingVerifications>()
                .expect("No verification hashmap found");
            let id = verifications.use_current_id();

            let embed = CreateEmbed::new()
                .title(format!("Mania profile for {}", user.username))
                .image(user.avatar_url)
                .description(format!(
                    "**- Country:** {country}\n
                        **- Rank:** Global: #{rank} | Country: #{country_rank}\n
                        https://osu.ppy.sh/users/{id}
                        ",
                    country = user.country.trim(),
                    rank = statistics.global_rank.unwrap_or(0),
                    country_rank = statistics.country_rank.unwrap_or(0),
                    id = user.user_id,
                ))
                .footer(CreateEmbedFooter::new(format!(
                    "[https://osu.ppy.sh/users/{}]",
                    user.user_id
                )))
                .color(0xCE7AFF);

            let status_embed = CreateEmbed::new()
                .title(format!(
                    "Verification Request for {}",
                    member.user.display_name()
                ))
                .description("**Current status:** 🟡 Pending");

            let button = CreateButton::new(id.to_string()).label("Click here to verify");

            let message = CreateMessage::new().embed(embed).button(button);
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
}
