use reqwest::{Client, Method, Response, Url};
use serde::Deserialize;
use serenity::prelude::TypeMapKey;
use std::str::FromStr;

pub struct Osu {
    client: Client,
    auth: OsuAuth,
}

#[derive(Deserialize)]
struct OsuAuth {
    access_token: String,
    expires_in: u32,
    token_type: String,
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

        println!("{:?}", request);

        let auth_raw = client.execute(request).await.ok()?;

        println!("{:?}", auth_raw);

        let auth = auth_raw.json().await.ok()?;

        Some(Self { client, auth })
    }

    pub async fn get_user(&self, user_id: &str) -> Option<Response> {
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

        self.client.execute(request).await.ok()
    }
}

impl TypeMapKey for Osu {
    type Value = Osu;
}

pub struct Quaver {
    client: Client,
}

impl TypeMapKey for Quaver {
    type Value = Quaver;
}
