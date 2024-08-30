use anyhow::Result;
use colored::Colorize;
use futures::future::join_all;
use futures::stream::{self, StreamExt};
use renai_client::client_ext::couchdb::Document;
use renai_client::prelude::Client as HttpClient;
use renai_client::prelude::*;
use renai_fs::schema::stocks::exe::StockDataset;
use renai_fs::schema::stocks::index::us::StockIndex;
use tokio_postgres::NoTls;

/// An object used in migrating .json data, from a local CouchDB database, to
/// a PostgreSQL database.
pub struct Migrator {
    http_client: HttpClient,
}

impl Migrator {
    /// Connect the Migrator to both the CouchDB and PostgreSQL databases.
    pub async fn connect() -> Result<Self> {
        Ok(Self {
            http_client: build_client(&std::env::var("USER_AGENT")?)?,
        })
    }

    /// Run all, fully-built migrations available.
    pub async fn migrate_all(&self, reset: &bool) -> Result<()> {
        self.migrate_stocks(reset).await?;
        Ok(())
    }

    /// Migrate the stock schema.
    pub async fn migrate_stocks(&self, reset: &bool) -> Result<()> {
        // fetch the index
        let base = std::env::var("COUCHDB_URL")?;
        let index_url = format!("{base}/stock/index");
        let index: Document<Vec<StockIndex>> =
            self.http_client.get(index_url).send().await?.json().await?;

        // if cli provides "renai migrate stocks -r / --reset"
        if reset == &true {
            // init postgres connection
            let (client, conn) =
                tokio_postgres::connect(&std::env::var("POSTGRES_URL").unwrap(), NoTls)
                    .await
                    .unwrap();

            // handle connection
            tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // pipeline initalising queries
            let queries = join_all([
                client.prepare("DROP SCHEMA IF EXISTS stock"),
                client.prepare("CREATE SCHEMA IF NOT EXISTS stock"),
                client.prepare("DROP TABLE IF EXISTS stock.index"),
                client.prepare("DROP TABLE IF EXISTS stock.price"),
                client.prepare("DROP TABLE IF EXISTS stock.metrics"),
                client.prepare(
                    "
                CREATE TABLE IF NOT EXISTS stock.index (
                    stock_id    CHAR(10) PRIMARY KEY,
                    ticker      VARCHAR(8),
                    title       VARCHAR(255)
                )",
                ),
                client.prepare(
                    "
                CREATE TABLE IF NOT EXISTS stock.price (
                    stock_id    CHAR(10),
                    dated       VARCHAR,
                    opening     FLOAT,
                    high        FLOAT,
                    low         FLOAT,
                    closing     FLOAT,
                    adj_close   FLOAT,
                    volume      INT
                )",
                ),
                client.prepare(
                    "
                CREATE TABLE IF NOT EXISTS stock.metrics (
                    stock_id    CHAR(10),
                    dated       VARCHAR,
                    metric      VARCHAR,
                    val         FLOAT
                )",
                ),
            ])
            .await;

            for query in queries {
                let _execution = client.execute(&query?, &[]).await;
            }

            log::info!("PostgreSQL tables for stocks initialised");
        }

        stream::iter(index.data)
            .for_each_concurrent(num_cpus::get(), |company| {
                let http_client = &self.http_client;
                let base: &String = &std::env::var("COUCHDB_URL")
                    .expect("failed to find environment variable: POSTGRES_URL");
                let url = format!("{}/stock/{}", base, &company.ticker);
                async move {
                    // retrieve stock price
                    let response = http_client.get(&url).send().await
                        .expect("failed to GET {url}");

                    match response.json::<Document<StockDataset>>().await {
                        Ok(stock) => {
                            // init postgres connection
                            let (client, conn) =
                                tokio_postgres::connect(&std::env::var("POSTGRES_URL").unwrap(), NoTls)
                                    .await
                                    .unwrap();

                            // handle connection
                            tokio::spawn(async move {
                                if let Err(e) = conn.await {
                                    eprintln!("connection error: {}", e);
                                }
                            });

                            let index_query = client.prepare("
                                INSERT INTO stock.index (stock_id, ticker, title)
                                VALUES ($1, $2, $3)
                            ")
                            .await
                            .expect("failed to unwrap index query");

                            let price_query = client.prepare("
                                INSERT INTO stock.price (stock_id, dated, opening, high, low, closing, adj_close, volume)
                                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                            ")
                            .await
                            .expect("failed to unwrap price query");

                            let metrics_query = client.prepare("
                                INSERT INTO stock.metrics (stock_id, dated, metric, val)
                                VALUES ($1, $2, $3, $4)
                            ")
                            .await
                            .expect("failed to unwrap metric query");

                            // insert the index
                            let _execute_index_query = &client
                                .query(
                                    &index_query,
                                    &[&company.cik_str, &company.ticker, &company.title],
                                )
                                .await
                                .map_err(|e| log::error!("{e}"));

                            // insert each price datacell
                            for row in stock.data.price {
                                let _execute_price_query = &client.query(
                                    &price_query,
                                    &[
                                        &company.cik_str,
                                        &row.dated,
                                        &row.open,
                                        &row.high,
                                        &row.low,
                                        &row.close,
                                        &row.adj_close,
                                        &row.volume
                                    ]
                                ).await.unwrap();
                            }

                            // insert each metric datacell
                            for row in stock.data.metrics {
                                // unpack `metrics: BTreeMap<String, f64>`
                                for record in row.metrics {
                                    let _execute_metric_query = &client
                                    .query(
                                        &metrics_query,
                                        &[&company.cik_str, &row.dated, &record.0, &record.1]
                                    ).await.unwrap();
                                }
                            }

                            // price inserts complete
                            log::info!(
                                "[{}] {} inserted to renai-pg",
                                &company.ticker, &company.title
                            );
                        },
                        Err(e) => log::error!("[{}] {} | {}", &company.ticker, &company.title, e.to_string().red()),
                    }
                }
            })
            .await;

        Ok(())
    }
}
