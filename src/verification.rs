use serenity::builder::{CreateEmbed, EditMember, EditMessage, EditRole};
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::config::Config;
use crate::user::User;

pub struct VerificationInfo {
    pub id: u32,
    pub discord_user: Member,
    pub user: User,
    pub status_message: Option<Message>,
    pub verification_message: Option<Message>,
}

impl VerificationInfo {
    pub async fn apply(&mut self, ctx: &Context, guild_id: &GuildId) -> Result<(), String> {
        let guild = match guild_id.to_partial_guild(&ctx.http).await {
            Ok(guild) => guild,
            Err(_) => return Err("Could not get server from id".to_string()),
        };

        let country = match self.user.country {
            Some(ref country) => country,
            None => return Err("Country has not been set".to_string()),
        };

        let country = crate::country_from_code(country).unwrap();

        let status_message = match self.status_message {
            Some(ref mut status_message) => status_message,
            None => return Err("Status message has not been created".to_string()),
        };

        let verification_message = match self.verification_message {
            Some(ref mut verification_message) => verification_message,
            None => return Err("Verification message has not been created".to_string()),
        };

        let mut emoji_shortcode = &country.to_lowercase().replace(" ", "_");

        let config = Config::load().unwrap_or_default();

        if let Some(exception) = config.emoji_exceptions.get(emoji_shortcode) {
            emoji_shortcode = exception;
        }

        let role_name = country.to_string()
            + " "
            + emojis::get_by_shortcode(emoji_shortcode)
                .ok_or(format!("Could not get emoji from country: {}", &country))?
                .as_str();

        let role = match guild.role_by_name(&role_name) {
            Some(role) => role,
            None => {
                // create role if it doesn't already exist
                let role_builder = EditRole::new().name(&role_name);
                &guild
                    .create_role(&ctx.http, role_builder)
                    .await
                    .map_err(|_| format!("Could not create new role: {role_name}"))?
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

        let member_settings = EditMember::new().nickname(&self.user.username);
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
        if let Err(_) = status_message.edit(&ctx.http, new_status).await {
            return Err(format!("Could not not edit status message"));
        }

        if let Err(_) = verification_message.delete(&ctx.http).await {
            return Err(format!("Failed to delete verification prompt"));
        }

        Ok(())
    }

    pub async fn deny(&mut self, ctx: &Context) -> Result<(), String> {
        let new_status_embed = CreateEmbed::new()
            .title("Verification Request")
            .description(format!(
                "**Current status for {}:** 🔴 Denied",
                self.discord_user.user.display_name()
            ));

        let new_status = EditMessage::new().embed(new_status_embed);

        let status_message = match self.status_message {
            Some(ref mut status_message) => status_message,
            None => return Err("Status message has not been created".to_string()),
        };

        let verification_message = match self.verification_message {
            Some(ref mut verification_message) => verification_message,
            None => return Err("Verification message has not been created".to_string()),
        };

        if let Err(_) = status_message.edit(&ctx.http, new_status).await {
            return Err(format!("Could not not edit status message"));
        }

        if let Err(_) = verification_message.delete(&ctx.http).await {
            return Err(format!("Failed to delete verification prompt"));
        }

        Ok(())
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

impl<'a> Deref for PendingVerifications {
    type Target = HashMap<u64, VerificationInfo>;

    fn deref(&self) -> &Self::Target {
        &self.verifications
    }
}

impl<'a> DerefMut for PendingVerifications {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.verifications
    }
}

impl TypeMapKey for PendingVerifications {
    type Value = PendingVerifications;
}
