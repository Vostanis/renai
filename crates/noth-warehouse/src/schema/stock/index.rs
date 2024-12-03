use crate::api::*;
use crate::schema::common::de_cik;
use async_trait::async_trait;
use dotenv::var;
use noth_util::{read_json, unzip, Util};
use serde::de::{MapAccess, Visitor};
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////
//
// API Documentation: https://www.sec.gov/search-filings/edgar-application-programming-interfaces
//
////////////////////////////////////////////////////////////////////////////////////////////////////

pub static INDEX_QUERY: &str = "
    INSERT INTO stock.index (stock_id, ticker, title, industry, nation)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (stock_id) DO NOTHING
";

pub static PRICE_QUERY: &str = "
    INSERT INTO stock.prices (stock_id, time, interval, opening, high, low, closing, adj_close, volume)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT (stock_id, time, interval) DO NOTHING
";

pub static METRIC_QUERY: &str = "
    INSERT INTO stock.metrics (stock_id, dated, metric, val, unit, taxonomy)
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (stock_id, dated, metric, val, unit, taxonomy) DO NOTHING
";

pub static FILINGS_QUERY: &str = "
    INSERT INTO stock.filings (stock_id, dated, filename, filetype, url, content, content_ts)
    VALUES ($1, $2, $3, $4, $5, $6, to_tsvector($6))
    ON CONFLICT (stock_id, filename) DO NOTHING
";

////////////////////////////////////////////////////////////////////////////////////////////////////
//
// SEC API
//
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Sec;

impl Sec {
    pub async fn scrape_index(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let http_client = Self::build_client();
        Self::etl(&http_client, pg_client).await?;
        Ok(())
    }

    fn build_client() -> HttpClient {
        reqwest::ClientBuilder::new()
            .user_agent(var("USER_AGENT").expect("failed to read USER_AGENT"))
            .build()
            .expect("failed to build reqwest client")
    }

    /// Download large files;
    ///     1. companyfacts.zip ~ 1.1GB
    ///     2. submissions.zip ~ 1.3GB
    pub async fn bulk_files(http_client: &reqwest::Client) -> anyhow::Result<()> {
        // 1. companyfacts.zip
        let url = "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
        let path = "./buffer/companyfacts.zip";
        debug!("downloading companyfacts.zip");
        match http_client.download_file(url, path).await {
            Ok(_) => trace!("downloaded companyfacts.zip"),
            Err(e) => {
                error!("failed to download companyfacts.zip, {}", e);
            }
        }
        debug!("downloaded companyfacts.zip");

        debug!("unzipping companyfacts.zip");
        unzip(path, "./buffer/companyfacts").await?;
        debug!("companyfacts.zip unzipped successfully");

        // 2. submissions.zip
        let url = "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";
        let path = "./buffer/submissions.zip";
        debug!("downloading submissions.zip");
        match http_client.download_file(url, path).await {
            Ok(_) => trace!("downloaded submissions.zip"),
            Err(e) => {
                error!("failed to download submissions.zip, {}", e);
            }
        }
        debug!("downloaded submissions.zip");

        debug!("unzipping submissions.zip");
        unzip(path, "./buffer/submissions").await?;
        debug!("submissions.zip unzipped successfully");

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

#[async_trait]
impl Api<Tickers> for Sec {
    async fn etl(http_client: &HttpClient, pg_client: &mut PgClient) -> anyhow::Result<()> {
        let url = "https://www.sec.gov/files/company_tickers.json".to_string();
        let tickers = Self::fetch(http_client, &url).await?;
        Self::insert(tickers, pg_client, ()).await?;
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

#[async_trait]
impl Http<Tickers> for Sec {
    async fn fetch(http_client: &HttpClient, url: &String) -> anyhow::Result<Tickers> {
        let tickers: Tickers = http_client.get(url).send().await?.json().await?;
        Ok(tickers)
    }
}

// -------------------------------------------------------------------------------------------------

#[async_trait]
impl Postgres<Tickers> for Sec {
    type Info = ();

    async fn insert(
        data: Tickers,
        pg_client: &mut PgClient,
        _info: Self::Info,
    ) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&INDEX_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(data.0);
        while let Some(cell) = stream.next().await {
            let path = format!("./buffer/submissions/CIK{}.json", cell.stock_id);
            trace!("reading file at path: \"{path}\"");
            let file: Sic = match read_json(&path).await {
                Ok(data) => data,
                Err(e) => {
                    error!("failed to read file | {e}");
                    continue;
                }
            };

            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &cell.stock_id,
                            &cell.ticker,
                            &cell.title,
                            &file.sic_description,
                            &"US",
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("Stock index inserted"),
                    Err(err) => error!("Failed to insert SEC Company Tickers | ERROR: {err}"),
                }
            }
            .await;
        }

        // unpack the transcation and commit it to the database
        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|e| {
                error!("failed to commit transaction for SEC Company Tickers");
                e
            })?;

        debug!(
            "Binance priceset inserted. Elapsed time: {} ms",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Deserialization
//
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Tickers(pub Vec<Ticker>);

// Individual stock behaviour; i.e., each ticker in the list needs to process price & metrics
// data (and any tertiary data) separately.
#[derive(Clone, Debug, Deserialize)]
pub struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    pub stock_id: String,
    pub ticker: String,
    pub title: String,
}

pub(crate) struct TickerVisitor;

impl<'de> Visitor<'de> for TickerVisitor {
    type Value = Tickers;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Map of tickers")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // each entry is in the form of:
        // `0: { "cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc." },
        //  1: { ... },
        //  ...`
        let mut tickers: Vec<Ticker> = Vec::new();
        while let Some((_, ticker)) = map.next_entry::<u16, Ticker>().expect("next_entry") {
            tickers.push(ticker);
        }
        Ok(Tickers(tickers))
    }
}

impl<'de> Deserialize<'de> for Tickers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // we want a vector returned, but the deserialize will expect a map, given
        // how the API has been designed
        deserializer.deserialize_map(TickerVisitor)
    }
}

// Struct for the SIC code retrived from submission files.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sic {
    sic_description: String,
}
