use anyhow::Result;
use renai_client::prelude::*;

#[allow(dead_code)]
pub struct Database {
    url: String,
    client: Client,
}

impl Database {
    /// Initiliase a `Database`, with a nested URL & Client.
    fn _new(url: String) -> Result<Self> {
        Ok(Self {
            url: url,
            client: build_client()?,
        })
    }

    /// Deploy the `Database` from a `docker-compose.yml` file.
    async fn _deploy() {}

    /// Stop the `Database`.
    async fn _stop() {}

    /// Find all the `fetch.rs` scripts, nested within the `schema/{dataset}` directory,
    /// and run them.
    /// 
    /// Each `fetch.rs` script will then handle ETL processes for each dataset.
    async fn _fetch() {
        // find all the fetch.rs scripts in schema/ and run them
    }
}