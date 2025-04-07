pub struct Osu {
    client: reqwest::Client,
}

impl OsuRequestHandler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn get_user(user_id: &str) {}
}

pub struct Quaver {}
