use super::SlashCommand;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;

pub struct ConfigCommand;

impl SlashCommand for ConfigCommand {
    fn register() -> CreateCommand {
        let channel_subcommand = CreateCommandOption::new(
            1.into(),
            "set_channel",
            "Set the channel used by this bot for a specific purpose",
        )
        .add_sub_option(CreateCommandOption::new(
            7.into(),
            "admin_only",
            "set the admin only channel",
        ))
        .add_sub_option(CreateCommandOption::new(
            7.into(),
            "verifications",
            "set the verifications channel",
        ));

        let config_command = CreateCommand::new("config")
            .description("Set the bot config")
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .add_option(channel_subcommand);

        config_command
    }

    async fn execute(ctx: &Context, cmd_data: &CommandData) -> String {
        let mut data = ctx.data.write().await;
        let config = data.get_mut::<Config>().expect("No config found");

        let mut response_buf = String::new();

        // nice indentation eh
        for option in cmd_data.options().iter() {
            if let ResolvedValue::SubCommand(scmds) = &option.value {
                for scmd in scmds.iter() {
                    if let ResolvedValue::Channel(channel) = scmd.value {
                        match scmd.name {
                            "admin_only" => {
                                config.admin_channel = Some(channel.id);
                                response_buf += format!(
                                    "Set admin only channel to {}\n",
                                    channel.name.clone().unwrap_or(channel.id.to_string())
                                )
                                .as_str();
                            }
                            "verifications" => {
                                config.verification_channel = Some(channel.id);
                                response_buf += format!(
                                    "Set verifications channel to {}\n",
                                    channel.name.clone().unwrap_or(channel.id.to_string())
                                )
                                .as_str();
                            }
                            _ => continue,
                        }
                    }
                }
            }
        }

        response_buf
    }
}
