use super::Args;
use serenity::builder::CreateCommand;
use serenity::model::prelude::*;
use serenity::prelude::*;

mod verify_command;
pub use verify_command::VerifyCommand;

mod config_command;
pub use config_command::ConfigCommand;

#[serenity::async_trait]
pub trait TextCommand {
    async fn execute_text(
        &self,
        ctx: &Context,
        channel_id: ChannelId,
        member: Member,
    ) -> Result<(), String>;
}

pub fn generate_command(args: Args) -> Option<impl TextCommand> {
    match args.cmd() {
        "verify" => Some(VerifyCommand::new(args.arg(0)?)),
        _ => None,
    }
}

pub trait SlashCommand {
    fn register() -> CreateCommand;
    fn execute(
        ctx: &Context,
        data: &CommandData,
    ) -> impl std::future::Future<Output = String> + Send;
}
