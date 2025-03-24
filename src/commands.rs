use super::Args;
use serenity::prelude::*;

#[serenity::async_trait]
pub trait Command {
    async fn execute(&self, ctx: Context);
}

pub struct VerifyCommand {
    osu_profile_url: String,
}

impl VerifyCommand {
    pub fn new(osu_profile_url: &str) -> Self {
        Self {
            osu_profile_url: osu_profile_url.to_string(),
        }
    }
}

#[serenity::async_trait]
impl Command for VerifyCommand {
    async fn execute(&self, ctx: Context) {
        todo!();
    }
}

pub fn generate_command(args: Args) -> Option<impl Command> {
    match args.cmd() {
        "verify" => Some(VerifyCommand::new(args.arg(0)?)),
        _ => None,
    }
}
