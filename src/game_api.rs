use reqwest::{Client, Method, Response, Url};
use serde::Deserialize;
use serenity::prelude::TypeMapKey;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct Osu {
    client: Client,
    client_id: String,
    client_secret: String,
    auth: OsuAuth,
    expires_in: Duration,
    refreshed_at: Instant,
}

#[derive(Deserialize)]
struct OsuAuth {
    access_token: String,
    expires_in: u64,
}

impl Osu {
    pub async fn build(
        client: Client,
        osu_client_id: &str,
        osu_client_secret: &str,
    ) -> Option<Self> {
        let url = Url::from_str("https://osu.ppy.sh/oauth/token").unwrap();

        let request_builder = client.request(Method::POST, url);

        let body = format!(
            "client_id={}&client_secret={}&grant_type=client_credentials&scope=public",
            osu_client_id, osu_client_secret
        );

        let request = request_builder
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(body)
            .build()
            .ok()?;

        let auth_raw = client.execute(request).await.ok()?;
        let auth: OsuAuth = auth_raw.json().await.ok()?;
        let expires_in = Duration::from_secs(auth.expires_in);

        Some(Self {
            client_id: osu_client_id.to_string(),
            client_secret: osu_client_secret.to_string(),
            client,
            auth,
            expires_in,
            refreshed_at: Instant::now(),
        })
    }

    async fn refresh_token(&mut self) {
        if let Some(osu) = Self::build(
            self.client.clone(),
            self.client_id.as_str(),
            self.client_secret.as_str(),
        )
        .await
        {
            self.auth = osu.auth;
            self.refreshed_at = Instant::now();
            self.expires_in = osu.expires_in
        }
    }

    pub async fn get_user(&mut self, user_id: &str) -> Option<Response> {
        let mut retries = 0;
        while retries < 3 {
            let api_url = Url::from_str(&format!(
                "https://osu.ppy.sh/api/v2/users/{}/mania",
                user_id
            ))
            .unwrap();

            let request_builder = self.client.request(Method::GET, api_url);

            let request = request_builder
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header(
                    "Authorization",
                    format!("Bearer {token}", token = self.auth.access_token),
                )
                .build()
                .unwrap();

            match self.client.execute(request).await {
                Ok(response) => return Some(response),
                Err(_) => {
                    if Instant::now().duration_since(self.refreshed_at) >= self.expires_in {
                        self.refresh_token().await;
                    }
                    retries += 1
                }
            }
        }
        return None;
    }
}

impl TypeMapKey for Osu {
    type Value = Osu;
}

pub struct Quaver;

impl Quaver {
    pub async fn get_user(user_id: &str) -> Option<Response> {
        let api_url = format!("https://api.quavergame.com/v2/user/{}", user_id);

        Some(reqwest::get(api_url).await.ok()?)
    }
}

pub struct Tachi;

impl Tachi {
    pub async fn get_user(user_id: &str) -> Option<Response> {
        let api_url = format!("https://boku.tachi.ac/api/v1/users/{}", user_id);

        Some(reqwest::get(api_url).await.ok()?)
    }

    pub async fn get_game_stats(user_id: &str, game: &str, playtype: &str) -> Option<Response> {
        let api_url = format!(
            "https://boku.tachi.ac/api/v1/users/{}/games/{}/{}",
            user_id, game, playtype
        );

        Some(reqwest::get(api_url).await.ok()?)
    }
}

impl TypeMapKey for Quaver {
    type Value = Quaver;
}
