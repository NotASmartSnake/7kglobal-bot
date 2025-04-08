use serenity::builder::{
    CreateButton, CreateEmbed, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::Args;
use crate::config::Config;
use crate::game_api::{DMJam, Osu, Quaver, Tachi};
use crate::user::{Game, User};
use crate::verification::{PendingVerifications, VerificationInfo};

const NOT_CONFIGURED: &'static str =
    "The bot is not yet configured, an admin needs to use the /config command";

async fn get_user_data(ctx: &Context, account: &str) -> Option<User> {
    if account.starts_with("https://osu.ppy.sh/users/") || account.starts_with("osu.ppy.sh/users/")
    {
        // write access so that the access token can be refreshed upon timeout
        let mut data = ctx.data.write().await;
        let osu = data.get_mut::<Osu>().unwrap();

        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "users" {
                let user_id = parts.next()?;
                let response = osu.get_user(user_id).await?;
                let response_text = response.text().await.ok()?;

                return Some(User::from_osu(&response_text));
            }
        }
        return None;
    }

    if account.starts_with("https://quavergame.com/user")
        || account.starts_with("quavergame.com/user")
    {
        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "user" {
                let user_id = parts.next()?;
                let response = Quaver::get_user(user_id).await?;
                let response_text = response.text().await.ok()?;

                return Some(User::from_quaver(&response_text));
            }
        }
    }

    if account.starts_with("https://boku.tachi.ac/u/")
        || account.starts_with("boku.tachi.ac/u/")
        || account.starts_with("https://bokutachi.xyz/u/")
        || account.starts_with("bokutachi.xyz/u/")
    {
        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "u" {
                let user_id = parts.next()?;

                let user_response = Tachi::get_user(user_id).await?;
                let user_response_text = user_response.text().await.ok()?;

                let game_stats_response = Tachi::get_game_stats(user_id, "bms", "7K").await?;
                let game_stats_response_text = game_stats_response.text().await.ok()?;

                return Some(User::from_tachi(
                    &user_response_text,
                    &game_stats_response_text,
                ));
            }
        }
    }
    if account.starts_with("https://dmjam.net/player-scoreboard/")
        || account.starts_with("dmjam.net/player-scoreboard/")
    {
        let mut parts = account.split("/");
        while let Some(part) = parts.next() {
            if part == "player-scoreboard" {
                let user_id = parts.next()?;
                let response = DMJam::get_user(user_id).await?;
                let response_text = response.text().await.ok()?;

                return Some(User::from_dmjam(&response_text));
            }
        }
    }
    None
}

fn create_profile_embed(user: &User, country: &str) -> CreateEmbed {
    let profile_type = match user.game {
        Game::Osu => "Mania",
        Game::Quaver => "Quaver 7k",
        Game::BMS => "BMS 7k",
        Game::DMJam => "DMJam",
    };

    CreateEmbed::new()
        .title(format!("{} profile for {}", profile_type, user.username))
        .image(user.avatar_url.clone())
        .description(format!(
            "**- Country:** {country}\n
                        **- Rank:** Global: #{rank} | Country: #{country_rank}\n
                        [{link}]
                        ",
            country = country,
            rank = user.ranks.global.unwrap_or(0),
            country_rank = user.ranks.country.unwrap_or(0),
            link = user.link,
        ))
        .color(0xCE7AFF)
}

async fn country_interaction(
    ctx: &Context,
    verification: &VerificationInfo,
    channel_id: &ChannelId,
) {
    let country_select = CreateSelectMenu::new(
        format!("GET-COUNTRY: {}", verification.id),
        CreateSelectMenuKind::Role {
            default_roles: None,
        },
    );
    let message = CreateMessage::new()
        .select_menu(country_select)
        .content(format!(
            "**{}, Select your country:**",
            verification.discord_user
        ));

    channel_id.send_message(&ctx.http, message).await.unwrap();
}

pub async fn verify_user(
    ctx: &Context,
    verification: &mut VerificationInfo,
    current_channel: &ChannelId,
    admin_channel: &ChannelId,
) -> Result<(), String> {
    let id = verification.id;

    let country = crate::country_from_code(
        &verification
            .user
            .country
            .clone()
            .ok_or("Country has not been set")?,
    )
    .ok_or("Country is not valid")?;

    let embed = create_profile_embed(&verification.user, country);

    let status_embed = CreateEmbed::new()
        .title(format!(
            "Verification Request for {}",
            verification.discord_user.user.display_name()
        ))
        .description("**Current status:** 🟡 Pending");

    let verify_button =
        CreateButton::new("verify ".to_string() + &id.to_string()).label("Click here to verify");

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
    let status_message = current_channel
        .send_message(&ctx.http, status_message)
        .await
        .expect("Failed to send status message");

    verification.status_message = Some(status_message);
    verification.verification_message = Some(message);

    Ok(())
}

pub async fn execute(
    ctx: &Context,
    channel_id: &ChannelId,
    member: Member,
    args: Args,
) -> Result<(), String> {
    let user = get_user_data(&ctx, &args.arg(0).ok_or("No argument supplied")?).await;

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

    let verifications = data
        .get_mut::<PendingVerifications>()
        .expect("No verification hashmap found");

    if let Some(user) = user {
        let id = verifications.use_current_id();

        let mut verification_info = VerificationInfo {
            id: id as u32,
            discord_user: member.clone(),
            user,
            status_message: None,
            verification_message: None,
        };

        if let Some(_) = verification_info.user.country {
            verify_user(&ctx, &mut verification_info, &channel_id, &admin_channel).await?;
            verifications.insert(id, verification_info);
        } else {
            country_interaction(&ctx, &verification_info, &channel_id).await;
            verifications.insert(id, verification_info);
        };
    };

    Ok(())
}
