pub mod commands;
pub mod config;
pub mod emoji_exceptions;
pub mod verification;

use config::Config;
use rosu_v2::prelude::*;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::verification::PendingVerifications;
use commands::{config_command, list_command, verify_command};

use std::env;
use std::str::FromStr;

pub struct GuildKey;

impl TypeMapKey for GuildKey {
    type Value = GuildId;
}

pub struct OsuKey;

impl TypeMapKey for OsuKey {
    type Value = Osu;
}

pub struct Args {
    cmd: String,
    args: Vec<String>,
}

impl Args {
    fn cmd(&self) -> &str {
        &self.cmd
    }

    fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|x| x.as_str())
    }
}

#[derive(Debug)]
pub struct ParseArgsError;

impl FromStr for Args {
    type Err = ParseArgsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<String> = s
            .trim()
            .to_lowercase()
            .split(" ")
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            cmd: s[0].clone(),
            args: s[1..].to_vec(),
        })
    }
}

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.content.len() == 0 {
            return;
        }

        if message.content.chars().collect::<Vec<char>>()[0] != '!' {
            return;
        }

        let args: Args = message.content[1..].parse().expect("This cannot fail");

        tokio::spawn(async move {
            if let Ok(member) = message.member(&ctx.http).await {
                let result = match args.cmd() {
                    "verify" => {
                        verify_command::execute(&ctx, message.channel_id, member, args).await
                    }
                    "list" => list_command::execute(&ctx, message.channel_id, args).await,
                    _ => return,
                };
                if let Err(e) = result {
                    if let Err(e) = message.channel_id.say(&ctx.http, e).await {
                        eprintln!("Could not send message {e}");
                    }
                }
            }
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let content = match command.data.name.as_str() {
                    "config" => config_command::execute(&ctx, &command.data).await,
                    _ => return,
                };

                let data = CreateInteractionResponseMessage::new().content(content);
                let response = CreateInteractionResponse::Message(data);
                if let Err(e) = command.create_response(&ctx.http, response).await {
                    eprintln!("Could not create response for interaction: {}", e);
                }
            }

            Interaction::Component(component) => {
                if let ComponentInteractionDataKind::Button = component.data.kind {
                    let mut data = ctx.data.write().await;
                    let pending_verifications = data
                        .get_mut::<PendingVerifications>()
                        .expect("No pending verification found in data");
                    let id = component.data.custom_id.clone();
                    let verification = pending_verifications
                        .get_mut(&id.parse::<u64>().expect("Invalid Id"))
                        .expect("Id could not be found in pending verifications");

                    let content = match verification.apply(&ctx).await {
                        Ok(()) => {
                            format!("Verified user: {}", &verification.discord_user.user.name)
                        }
                        Err(e) => e,
                    };

                    let data = CreateInteractionResponseMessage::new().content(content);
                    let response = CreateInteractionResponse::Message(data);
                    if let Err(e) = component.create_response(&ctx.http, response).await {
                        eprintln!("Could not create response for interaction: {}", e);
                    }
                }
            }
            _ => println!("Not yet implemented"),
        }
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        println!("session with id: {} started", data_about_bot.session_id);

        let commands = vec![commands::config_command::register()];

        {
            let data = ctx.data.read().await;
            let guild_id = data.get::<GuildKey>().expect("No guild key found");

            let _ = guild_id.set_commands(&ctx.http, commands).await;
        }

        let config = match Config::load_from_file("config.json") {
            Some(config) => config,
            None => Config::default(),
        };

        let mut data = ctx.data.write().await;
        data.insert::<Config>(config);
    }
}
