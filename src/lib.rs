use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::Command;

use std::str::FromStr;

mod commands;

struct Args {
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
struct ParseArgsError;

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
        if message.content.chars().collect::<Vec<_>>()[0] != '!' {
            return;
        }

        let args: Args = message.content[1..].parse().unwrap();

        let command = commands::generate_command(args);
        if let Some(command) = command {
            tokio::spawn(async move {
                command.execute(ctx).await;
            });
        }
    }
}
