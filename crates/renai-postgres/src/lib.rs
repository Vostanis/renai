pub mod schema;

use anyhow::Result;
use renai_client::prelude::Client as HttpClient;
use renai_client::prelude::*;
use sqlx::PgPool;

/// An object used in migrating .json data, from a local CouchDB database, to
/// a PostgreSQL database.
pub struct Migrator {
    http_client: HttpClient,
    pg_pool: PgPool,
}

impl Migrator {
    /// Connect the Migrator to both the CouchDB and PostgreSQL databases.
    pub async fn connect() -> Result<Self> {
        Ok(Self {
            http_client: build_client()?,
            // http_client: HttpClient::connect(&std::env::var("COUCHDB_URL")),
            pg_pool: PgPool::connect(&std::env::var("POSTGRES_URL")?).await?,
        })
    }

    /// Run all, fully-built migrations available.
    pub async fn migrate_all(&self) -> Result<()> {
        self.migrate_stocks().await?;

        Ok(())
    }
    
    /// Migrate the stock schema.
    pub async fn migrate_stocks(&self) -> Result<()> {
        let base = std::env::var("COUCHDB_URL")?;
        let index_url = format!("{base}/stock/index");
        let _index = self.http_client
            .get(index_url)
            .send()
            .await?;

        // async loop the index; pipeline tokio_postres queries

        // how do i have a table with any number of columns?

        Ok(())
    }
}