use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use std::fmt::Debug;
use tokio_postgres::Client as PgClient;
use tracing::error;

/// ETL framework.
///
/// Because of the large number of API-scraping, the API is split into two parts in order to
/// segment code neatly (effectively, this is purely organisational);
///
///     1. `[Http]` - the procedure for fetching data type `T` from some HTTP endpoint.
///     2. `[Postgres]` - the procedure for inserting the data type `T` into a PostgreSQL database.
#[async_trait]
pub trait Api<T>: Postgres<T> + Http<T>
where
    T: Debug + Send + Sync,
{
    /// Shortcut method for the entire API-scraping process.
    async fn etl(http_client: &HttpClient, pg_client: &mut PgClient) -> Result<()>;
}

/// API to the HTTP endpoint data type `T`; how is the data **extracted** and **transformed**?
#[async_trait]
pub trait Http<T>
where
    T: Debug + Send + Sync,
{
    /// How the data type `T` is fetched from some HTTP endpoint.
    async fn fetch(http_client: &HttpClient, url: &String) -> Result<T>;

    /// Pre-defined `fetch()` for when `serde::Deserialize` is defined to handle transformations under the
    /// hood.
    async fn fetch_de<D>(http_client: &HttpClient, url: &String) -> Result<D>
    where
        D: serde::de::DeserializeOwned,
    {
        let response = http_client.get(url).send().await.map_err(|e| {
            error!("failed fetching response from {url}");
            e
        })?;

        let de: D = response.json().await.map_err(|e| {
            error!("failed deserializing from {url}");
            e
        })?;

        Ok(de)
    }
}

/// API to a PostgreSQL database, in which the `Data` will be inserted into; how is the data
/// **loaded**?
#[async_trait]
pub trait Postgres<T>
where
    T: Debug + Send + Sync,
{
    type Info;

    /// How the `Data` is inserted into the PostgreSQL database.
    async fn insert(data: T, pg_client: &mut PgClient, info: Self::Info) -> Result<()>;
}
