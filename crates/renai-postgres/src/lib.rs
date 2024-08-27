use anyhow::Result;
// use futures::StreamExt;
use renai_client::prelude::Client as HttpClient;
use renai_client::prelude::*;
use renai_client::client_ext::couchdb::Document;
use renai_fs::schema::stocks::exe::StockDataset;
use renai_fs::schema::stocks::index::us::StockIndex;
use tokio_postgres::{NoTls, Statement};
use tokio_stream::StreamExt;

/// An object used in migrating .json data, from a local CouchDB database, to
/// a PostgreSQL database.
pub struct Migrator {
    http_client: HttpClient,
    // pg_pool: PgPool,
}

impl Migrator {
    /// Connect the Migrator to both the CouchDB and PostgreSQL databases.
    pub async fn connect() -> Result<Self> {
        Ok(Self {
            http_client: build_client(&std::env::var("USER_AGENT")?)?,
            // pg_pool: PgPool::connect(&std::env::var("POSTGRES_URL")?).await?,
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
        let index: Document<Vec<StockIndex>>  = self.http_client
            .get(index_url)
            .send()
            .await?
            .json()
            .await?;

        // async loop the index, and insert each row for each stock
        let mut stream = tokio_stream::iter(index.data);
        while let Some(company) = stream.next().await {
            // fetch the individual stock dataset
            let base = base.clone();
            let url = format!("{}/stock/{}", &base, &company.ticker);
            let stock: Document<StockDataset> = self.http_client
                .get(url)
                .send()
                .await?
                .json()
                .await?;                

            // init postgres connectionlet (client, connection) =
            let (client, conn) = 
                tokio_postgres::connect(&std::env::var("POSTGRES_URL")?, NoTls).await?;

            // handle connection
            tokio::spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // insert the index
            let _index_query = &client.query("
                INSERT INTO stocks.index (pk_stocks, ticker, title)
                VALUES ($1, $2, $3)",
                &[
                    &company.cik_str,
                    &company.ticker,
                    &company.title
                ]
            ).await?;
            println!("[{}] {} inserted into stocks.index", &company.ticker, &company.title);

            // insert each price datacell
            let mut stream = tokio_stream::iter(stock.data.price);
            while let Some(cell) = stream.next().await {
                let _price_query = &client.query(
                "
                    INSERT INTO stocks.price (pk_stocks, dated, opening, high, low, closing, adj_close, volume)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    &[
                        &company.cik_str,
                        &cell.dated,
                        &cell.open,
                        &cell.high,
                        &cell.low,
                        &cell.close,
                        &cell.adj_close,
                        &cell.volume
                    ]
                ).await?;
            }
            println!("[{}] {} inserted to stocks.price", &company.ticker, &company.title);
        };

        // how do i have a table with any number of columns?

        Ok(())
    }
}