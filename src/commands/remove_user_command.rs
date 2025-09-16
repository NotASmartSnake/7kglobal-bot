use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::*;

use rusqlite::{params, Connection};

pub fn register() -> CreateCommand {
    let game = CreateCommandOption::new(3.into(), "game", "Set the game of the user registration").required(true);

    let username = CreateCommandOption::new(3.into(), "username", "Set the username of the user registration").required(true);

    CreateCommand::new("remove_user")
        .description("Remove a user from the database")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(game)
        .add_option(username)
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveUserError {
    InvalidGame(String),
    DatabaseFailure,
    InvalidOption
}

pub async fn execute(cmd_data: &CommandData) -> Result<String, RemoveUserError> {
    let game = cmd_data.options().iter().find(|option| option.name == "game")
        .ok_or(RemoveUserError::InvalidOption)?
        .value.clone();

    let username = cmd_data.options().iter().find(|option| option.name == "username")
        .ok_or(RemoveUserError::InvalidOption)?
        .value.clone();

    let game = if let ResolvedValue::String(game_str) = game {
        game_str
    } else {
        return Err(RemoveUserError::InvalidOption);
    };

    let username = if let ResolvedValue::String(username_str) = username {
        username_str
    } else {
        return Err(RemoveUserError::InvalidOption);
    };

    let conn = Connection::open("users.db").map_err(|_| RemoveUserError::DatabaseFailure)?;

    conn.execute("DELETE FROM users WHERE game=?1 AND username=?2", params![game, username])
        .map_err(|_| RemoveUserError::DatabaseFailure)?;

    Ok(format!("Successfully removed {username} from the database"))
}
