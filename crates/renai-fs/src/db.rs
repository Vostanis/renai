use anyhow::Result;
use renai_client::prelude::*;

use crate::schema2;

#[allow(dead_code)]
pub struct Database {
    url: String,
    client: Client,
}

impl Database {
    /// Initiliase a `Database`, with a nested URL & Client.
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            url: url.to_string(),
            client: build_client()?,
        })
    }

    /// Deploy the `Database` from a `docker-compose.yml` file.
    async fn _deploy() {}

    /// Find all the `fetch.rs` scripts, nested within the `schema/{dataset}` directory,
    /// and run them.
    /// 
    /// Each `fetch.rs` script will then handle ETL processes for each dataset.
    /// 
    /// ```rust
    /// let db = Database::new(".env");
    /// db.fetch([
    ///     "crypto",
    ///     "economic",
    ///     "people",
    ///     "stocks",
    /// ]).await?;
    /// ```
    pub async fn fetch(&self, args: Vec<&str>) -> Result<()> {
        // find all the schematic.rs scripts in schema/{args[i]} and run them
        for arg in args {
            match arg {
                "stocks" => schema2::stocks::exe::exe(&self.client).await?,
                _ => unreachable!()
            }
        }

        Ok(())
    }

    /// Stop the `Database`.
    async fn _stop() {}
}