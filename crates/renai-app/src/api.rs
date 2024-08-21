use anyhow::Result;
use serde::{Deserialize, Serialize};

const DATABASE_URL: &'static str = "http://admin:password@localhost:5984";

pub struct User {
    client: reqwest::Client,
    metadata: UserMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct UserMetadata {
    id: usize,
    username: String,
    password: String,
}

impl User {
    fn new(id: usize, username: String, password: String) -> Self {
        let metadata = UserMetadata {
            id,
            username,
            password,
        };

        User {
            client: reqwest::Client::new(),
            metadata
        }
    }

    async fn fetch_ticker(&self, ticker: String) -> Result<Option<String>> {
        let client = &self.client;
        let url = format!("{DATABASE_URL}/stock/{ticker}");
        let response = client.get(url)
            .send()
            .await?
            .text()
            .await?;
        Ok(Some(response))
    }
}