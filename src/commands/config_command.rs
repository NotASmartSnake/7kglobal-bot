use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::config::Config;

pub fn register() -> CreateCommand {
    let set_channel = CreateCommandOption::new(
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

    let add_emoji_exception =
        CreateCommandOption::new(1.into(), "add_emoji_exception", "add an emoji exception")
            .add_sub_option(
                CreateCommandOption::new(3.into(), "country", "name of the country in lowercase")
                    .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    3.into(),
                    "shortcode",
                    "shortcode for the emoji, using the github shortcodes",
                )
                .required(true),
            );

    let remove_emoji_exception = CreateCommandOption::new(
        1.into(),
        "remove_emoji_exception",
        "remove a country from emoji exceptions",
    )
    .add_sub_option(
        CreateCommandOption::new(3.into(), "country", "name of the country").required(true),
    );

    let config_command = CreateCommand::new("config")
        .description("Set the bot config")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(add_emoji_exception)
        .add_option(remove_emoji_exception)
        .add_option(set_channel);

    config_command
}

pub async fn execute(cmd_data: &CommandData) -> String {
    let mut config = match Config::load() {
        Some(config) => config,
        None => Config::default(),
    };

    let mut response_buf = String::new();

    for option in cmd_data.options().iter() {
        match option.name {
            "set_channel" => set_channel(&mut config, &option.value, &mut response_buf),
            "add_emoji_exception" => {
                add_emoji_exception(&mut config, &option.value, &mut response_buf)
            }
            "remove_emoji_exception" => {
                remove_emoji_exception(&mut config, &option.value, &mut response_buf)
            }
            _ => response_buf += format!("{} is not a valid option", option.name).as_str(),
        }
    }

    if let Err(e) = config.save() {
        response_buf += &e;
    }

    response_buf
}

fn add_emoji_exception(config: &mut Config, cmd_value: &ResolvedValue, response_buf: &mut String) {
    if let ResolvedValue::SubCommand(scmds) = cmd_value {
        let country = if let ResolvedValue::String(country) = scmds[0].value {
            country
        } else {
            return;
        };

        let shortcode = if let ResolvedValue::String(shortcode) = scmds[1].value {
            shortcode
        } else {
            return;
        };

        config
            .emoji_exceptions
            .insert(country.to_string(), shortcode.to_string());

        *response_buf += format!("Added {} -> {} to emoji exceptions", country, shortcode).as_str();
    }
}

fn remove_emoji_exception(
    config: &mut Config,
    cmd_value: &ResolvedValue,
    response_buf: &mut String,
) {
    if let ResolvedValue::String(input) = cmd_value {
        *response_buf += match config.emoji_exceptions.remove(*input) {
            Some(_) => "Removed country from emoji exceptions",
            None => "Country was not in emoji exceptions",
        }
    }
}

fn set_channel(config: &mut Config, cmd_value: &ResolvedValue, response_buf: &mut String) {
    if let ResolvedValue::SubCommand(scmds) = cmd_value {
        for scmd in scmds.iter() {
            if let ResolvedValue::Channel(channel) = scmd.value {
                match scmd.name {
                    "admin_only" => {
                        config.channels.admin_channel = Some(channel.id);
                        *response_buf += format!(
                            "Set admin only channel to {}\n",
                            channel.name.clone().unwrap_or(channel.id.to_string())
                        )
                        .as_str();
                    }
                    "verifications" => {
                        config.channels.verification_channel = Some(channel.id);
                        *response_buf += format!(
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
