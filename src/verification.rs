use serenity::builder::{CreateEmbed, EditMember, EditMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::{GuildKey, emoji_exceptions};

pub struct VerificationInfo {
    pub discord_user: Member,
    pub osu_username: String,
    pub country: String,
    pub status_message: Message,
    pub verification_message: Message,
}

impl VerificationInfo {
    pub async fn apply(&mut self, ctx: &Context) -> Result<(), String> {
        let data = ctx.data.read().await;
        let guild_id = data.get::<GuildKey>().expect("No guild key found");

        let guild = match guild_id.to_partial_guild(&ctx.http).await {
            Ok(guild) => guild,
            Err(_) => return Err("Could not get server from id".to_string()),
        };

        let mut emoji_shortcode = &self.country.to_lowercase().replace(" ", "_");

        let exceptions = emoji_exceptions::get_emoji_exceptions();

        if let Some(exception) = exceptions.get(emoji_shortcode) {
            emoji_shortcode = exception;
        }

        println!("{}", emoji_shortcode);

        let role = match guild.role_by_name(
            &(self.country.clone()
                + " "
                + emojis::get_by_shortcode(emoji_shortcode)
                    .ok_or(format!(
                        "Could not get emoji from country: {}",
                        &self.country
                    ))?
                    .as_str()),
        ) {
            Some(role) => role,
            None => {
                return Err(format!(
                    "There is currently no role set up for the country: {}",
                    &self.country
                ));
            }
        };

        if let Err(e) = self.discord_user.add_role(&ctx.http, role).await {
            return Err(format!("Could not add role to user: {e}"));
        }

        let member_role = match guild.role_by_name("Member") {
            Some(role) => role,
            None => return Err("Member role does not exist".to_string()),
        };

        if let Err(e) = self.discord_user.add_role(&ctx.http, member_role).await {
            return Err(format!("Could not add role to user: {e}"));
        }

        let member_settings = EditMember::new().nickname(&self.osu_username);
        if let Err(e) = self.discord_user.edit(&ctx.http, member_settings).await {
            return Err(format!("Could not edit the users' nickname: {e}"));
        }

        let new_status_embed = CreateEmbed::new()
            .title("Verification Request")
            .description(format!(
                "**Current status for {}:** 🟢 Accepted",
                self.discord_user.user.display_name()
            ));

        let new_status = EditMessage::new().embed(new_status_embed);
        if let Err(_) = self.status_message.edit(&ctx.http, new_status).await {
            return Err(format!("Could not not edit status message"));
        }

        if let Err(_) = self.verification_message.delete(&ctx.http).await {
            return Err(format!("Failed to delete verification prompt"));
        }

        Ok(())
    }

    pub async fn deny(&self, ctx: &Context) {
        let new_status_embed = CreateEmbed::new()
            .title("Verification Request")
            .description(format!(
                "**Current status for {}:** 🟢 Denied",
                self.discord_user.user.display_name()
            ));
    }
}

#[derive(Default)]
pub struct PendingVerifications {
    current_id: u64,
    verifications: HashMap<u64, VerificationInfo>,
}

impl PendingVerifications {
    pub fn use_current_id(&mut self) -> u64 {
        let id = self.current_id;
        self.current_id += 1;
        id.clone()
    }
}

impl Deref for PendingVerifications {
    type Target = HashMap<u64, VerificationInfo>;

    fn deref(&self) -> &Self::Target {
        &self.verifications
    }
}

impl DerefMut for PendingVerifications {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.verifications
    }
}

impl TypeMapKey for PendingVerifications {
    type Value = PendingVerifications;
}
