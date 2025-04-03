use serde::Deserialize;
use std::collections::HashMap;

pub enum Game {
    Osu,
    Quaver,
}

#[derive(Deserialize)]
pub struct Ranks {
    global: u32,
    country: u32,
}

pub struct User {
    pub game: Game,
    pub username: String,
    pub country: String,
    pub ranks: Ranks,
    pub avatar_url: String,
}

#[derive(Deserialize)]
struct OsuCountry {
    pub code: String,
}

impl User {
    pub fn from_osu(response: &str) -> Option<Self> {
        let response = serde_json::from_str::<HashMap<String, String>>(response).unwrap();

        let username = response.get("username")?;
        let country = serde_json::from_str::<OsuCountry>(response.get("country")?)
            .unwrap()
            .code;
        let ranks = serde_json::from_str::<Ranks>(response.get("rank")?).unwrap();
        let avatar_url = response.get("avatar_url")?;

        Some(Self {
            game: Game::Osu,
            username: username.to_string(),
            country,
            ranks,
            avatar_url: avatar_url.to_string(),
        })
    }
    pub fn from_quaver(response: &str) -> Option<Self> {
        let response = serde_json::from_str::<HashMap<String, String>>(response).unwrap();
        let stats_7k =
            serde_json::from_str::<HashMap<String, String>>(response.get("stats_keys7")?).unwrap();

        let username = response.get("username")?;
        let avatar_url = response.get("avatar_url")?;
        let country = response.get("country")?;

        let ranks = serde_json::from_str::<Ranks>(stats_7k.get("ranks")?).unwrap();

        Some(Self {
            game: Game::Quaver,
            username: username.to_string(),
            country: country.to_string(),
            avatar_url: avatar_url.to_string(),
            ranks,
        })
    }
}
