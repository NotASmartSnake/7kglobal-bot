use serde::Deserialize;

pub enum Game {
    Osu,
    Quaver,
    BMS,
}

#[derive(Deserialize, Debug)]
pub struct Ranks {
    pub global: Option<u32>,
    pub country: Option<u32>,
}

pub struct User {
    pub game: Game,
    pub username: String,
    pub country: Option<String>,
    pub ranks: Ranks,
    pub avatar_url: String,
    pub link: String,
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

impl User {
    pub fn from_osu(response: &str) -> Self {
        let response = serde_json::from_str::<OsuUser>(response).unwrap();
        let link = format!("http://osu.ppy.sh/users/{}", response.id);

        let ranks = Ranks {
            global: response.statistics.global_rank,
            country: response.statistics.country_rank,
        };

        Self {
            game: Game::Osu,
            username: response.username.to_string(),
            avatar_url: response.avatar_url.to_string(),
            country: Some(response.country.code.to_string()),
            ranks,
            link,
        }
    }

    pub fn from_quaver(response: &str) -> Self {
        let response = serde_json::from_str::<QuaverUserResponse>(response)
            .unwrap()
            .user;
        let link = format!("https://quavergame.com/user/{}", response.id);

        Self {
            game: Game::Quaver,
            username: response.username.to_string(),
            country: Some(response.country.to_string()),
            avatar_url: response.avatar_url.to_string(),
            ranks: response.stats_keys7.ranks,
            link,
        }
    }

    pub fn from_tachi(user_response: &str, user_game_stats_response: &str) -> Self {
        println!("{}", user_response);
        let user_response = serde_json::from_str::<TachiUserResponse>(user_response)
            .unwrap()
            .body;
        println!("{}", user_game_stats_response);
        let user_game_stats_response =
            serde_json::from_str::<TachiGameStatsResponse>(user_game_stats_response)
                .unwrap()
                .body;

        let link = format!(
            "https://boku.tachi.ac/u/{}",
            user_response.username_lowercase
        );

        let ranks = Ranks {
            global: Some(user_game_stats_response.ranking_data.sieglinde.ranking),
            country: None,
        };

        Self {
            game: Game::BMS,
            username: user_response.username,
            country: None,
            avatar_url: format!(
                "https://boku.tachi.ac/api/v1/users/{}/pfp",
                user_response.id
            ),
            link,
            ranks,
        }
    }
}
