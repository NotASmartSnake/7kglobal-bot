use rusqlite::{Connection, params};
use serde::Deserialize;
use serenity::builder::CreateEmbed;
use std::fmt;
use std::str::FromStr;

pub enum Game {
    Osu,
    Quaver,
    BMS,
    DMJam,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Osu => write!(f, "osu"),
            Self::Quaver => write!(f, "quaver"),
            Self::BMS => write!(f, "bms"),
            Self::DMJam => write!(f, "dmjam"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseGameError;

impl FromStr for Game {
    type Err = ParseGameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "osu" => Ok(Self::Osu),
            "quaver" => Ok(Self::Quaver),
            "bms" => Ok(Self::BMS),
            "dmjam" => Ok(Self::DMJam),
            _ => Err(ParseGameError),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Ranks {
    pub global: Option<u32>,
    pub country: Option<u32>,
}

pub struct User {
    pub game: Game,
    pub user_id: u32,
    pub username: String,
    pub country: Option<String>,
    pub ranks: Ranks,
    pub avatar_url: String,
    pub link: String,
    pub playtime: Option<u32>,
    pub level: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct OsuUser {
    pub username: String,
    pub country: OsuCountry,
    pub statistics: OsuUserStatistics,
    pub avatar_url: String,
    pub id: u32,
}

#[derive(Deserialize, Debug)]
struct OsuUserStatistics {
    pub global_rank: Option<u32>,
    pub country_rank: Option<u32>,
    pub play_time: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct OsuCountry {
    pub code: String,
}

#[derive(Deserialize, Debug)]
struct QuaverUserResponse {
    pub user: QuaverUser,
}

#[derive(Deserialize, Debug)]
struct QuaverUser {
    pub id: u32,
    pub username: String,
    pub avatar_url: String,
    pub stats_keys7: QuaverUserStatistics,
    pub country: String,
}

#[derive(Deserialize, Debug)]
struct QuaverUserStatistics {
    pub ranks: Ranks,
}

#[derive(Deserialize, Debug)]
struct TachiUserResponse {
    body: TachiUser,
}

#[derive(Deserialize, Debug)]
struct TachiGameStatsResponse {
    body: TachiGameStats,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TachiUser {
    pub id: u32,
    pub username: String,
    pub username_lowercase: String,
    pub playtime: Option<u64>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TachiGameStats {
    pub ranking_data: TachiStupidSection,
}

#[derive(Deserialize, Debug)]
struct TachiStupidSection {
    pub sieglinde: TachiRankingData,
}

#[derive(Deserialize, Debug)]
struct TachiRankingData {
    pub ranking: u32,
}

#[derive(Deserialize, Debug)]
struct DMJamUser {
    player_code: u32,
    nickname: String,
    player_ranking: u32,
    level: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DBSaveError;

impl User {
    pub fn save_to_database(
        &self,
        discord_user_id: u64,
        country: Option<&str>,
    ) -> Result<(), DBSaveError> {
        let conn = Connection::open("users.db").map_err(|_| DBSaveError)?;

        let _ = conn.execute(
            "INSERT INTO users (discord_id, game, player_id, username, country) values (?1, ?2, ?3, ?4, ?5)",
            params![
                discord_user_id,
                self.game.to_string(),
                self.user_id,
                self.username,
                country.map(|s| s.to_string())
            ],
        );

        Ok(())
    }

    pub fn from_osu(response: &str) -> Option<Self> {
        let response = serde_json::from_str::<OsuUser>(response).ok()?;
        let link = format!("http://osu.ppy.sh/users/{}", response.id);

        let ranks = Ranks {
            global: response.statistics.global_rank,
            country: response.statistics.country_rank,
        };

        let playtime = if let Some(playtime) = response.statistics.play_time {
            Some((playtime / 3600 as u64) as u32)
        } else {
            None
        };

        Some(Self {
            game: Game::Osu,
            user_id: response.id,
            username: response.username.to_string(),
            avatar_url: response.avatar_url.to_string(),
            country: Some(response.country.code.to_string()),
            ranks,
            link,
            playtime,
            level: None,
        })
    }

    pub fn from_quaver(response: &str) -> Option<Self> {
        let response = serde_json::from_str::<QuaverUserResponse>(response)
            .ok()?
            .user;
        let link = format!("https://quavergame.com/user/{}", response.id);

        Some(Self {
            game: Game::Quaver,
            user_id: response.id,
            username: response.username.to_string(),
            country: Some(response.country.to_string()),
            avatar_url: response.avatar_url.to_string(),
            ranks: response.stats_keys7.ranks,
            link,
            playtime: None,
            level: None,
        })
    }

    pub fn from_tachi(user_response: &str, user_game_stats_response: &str) -> Option<Self> {
        let user_response = serde_json::from_str::<TachiUserResponse>(user_response)
            .ok()?
            .body;

        let user_game_stats_response =
            serde_json::from_str::<TachiGameStatsResponse>(user_game_stats_response)
                .ok()?
                .body;

        let link = format!(
            "https://boku.tachi.ac/u/{}",
            user_response.username_lowercase
        );

        let ranks = Ranks {
            global: Some(user_game_stats_response.ranking_data.sieglinde.ranking),
            country: None,
        };

        let playtime = if let Some(playtime) = user_response.playtime {
            Some((playtime / 3e6 as u64) as u32)
        } else {
            None
        };

        Some(Self {
            game: Game::BMS,
            user_id: user_response.id,
            username: user_response.username,
            country: None,
            avatar_url: format!(
                "https://boku.tachi.ac/api/v1/users/{}/pfp",
                user_response.id
            ),
            link,
            ranks,
            playtime,
            level: None,
        })
    }

    pub fn from_dmjam(response: &str) -> Option<Self> {
        let response = serde_json::from_str::<DMJamUser>(response).ok()?;

        let ranks = Ranks {
            global: Some(response.player_ranking),
            country: None,
        };

        let link = format!(
            "https://dmjam.net/player-scoreboard/{}/2",
            response.player_code
        );

        Some(Self {
            game: Game::DMJam,
            user_id: response.player_code,
            username: response.nickname,
            avatar_url: String::new(),
            ranks,
            country: None,
            link,
            playtime: None,
            level: Some(response.level),
        })
    }

    pub fn create_profile_embed(&self, country: &str) -> CreateEmbed {
        match self.game {
            Game::Osu => Self::create_osu_embed(&self, country),
            Game::Quaver => Self::create_quaver_embed(&self, country),
            Game::DMJam => Self::create_dmjam_embed(&self, country),
            Game::BMS => Self::create_bokutachi_embed(&self, country),
        }
    }

    fn create_osu_embed(user: &User, country: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(format!("Osu profile for {}", user.username))
            .image(user.avatar_url.clone())
            .description(format!(
                "**- Country:** {country}\n
                **- Rank:** Global: #{rank} | Country: #{country_rank}\n
                **- Play Time:** {playtime}h\n
                [{link}]
                ",
                country = country,
                rank = user.ranks.global.unwrap_or(0),
                country_rank = user.ranks.country.unwrap_or(0),
                playtime = user.playtime.unwrap_or(0),
                link = user.link,
            ))
            .color(0xff66f0)
    }

    fn create_quaver_embed(user: &User, country: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(format!("Quaver 7k profile for {}", user.username))
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
            .color(0xff66f0)
    }

    fn create_bokutachi_embed(user: &User, country: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(format!("BMS 7k profile for {}", user.username))
            .description(format!(
                "**- Country:** {country}\n
                **- Rank:** #{rank}\n
                **- Play Time:** {playtime}h\n
                [{link}]
                ",
                country = country,
                rank = user.ranks.global.unwrap_or(0),
                playtime = user.playtime.unwrap_or(0),
                link = user.link,
            ))
            .color(0xff66f0)
    }

    fn create_dmjam_embed(user: &User, country: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(format!("DMJam profile for {}", user.username))
            .description(format!(
                "**- Country:** {country}\n
                **- Level:** {level}\n
                **- Rank:** #{rank}\n
                [{link}]
                ",
                country = country,
                level = user.level.unwrap_or(0),
                rank = user.ranks.global.unwrap_or(0),
                link = user.link,
            ))
            .color(0xff66f0)
    }
}
