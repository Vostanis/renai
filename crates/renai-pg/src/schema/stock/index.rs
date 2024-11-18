#![allow(dead_code)]

use crate::fs::read_json;
use crate::schema::common::de_cik;
use crate::schema::stock::{metrics, prices};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use std::collections::HashSet as Set;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////
// API Documentation: https://www.sec.gov/search-filings/edgar-application-programming-interfaces
////////////////////////////////////////////////////////////////////////////////////////////////////

kvapi::api! {
    name: Sec
    base: "https://www.sec.gov/files/"
    head: { "User-Agent": &dotenv::var("USER_AGENT")? }
    dict: { "company_tickers.json" -> Tickers }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// US Stock Tickers/Titles

/// Collect full data of the entire list of US stocks.
#[derive(Debug)]
pub struct Tickers(pub Vec<Ticker>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sic {
    sic_description: String,
}

impl Tickers {
    pub async fn fetch(http_client: &reqwest::Client) -> anyhow::Result<Self> {
        let tickers: Self = http_client
            .get("https://www.sec.gov/files/company_tickers.json")
            .send()
            .await?
            .json()
            .await?;
        Ok(tickers)
    }

    pub async fn insert(&self, pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
        let query = pg_client
            .prepare(
                "
            INSERT INTO stock.index (stock_id, ticker, title, industry, nation)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT stock_id DO NOTHING",
            )
            .await?;

        let transaction = Arc::new(pg_client.transaction().await?);

        // BUG: duplicate entries are being inserted into the transaction and failing.
        //      No built-in way of error-handling.
        //
        // FIX: build a hashset to avoid duplicates; avoiding async for a small vec
        let mut set = Set::<&String>::new();

        // load to `stock.prices`
        let time = std::time::Instant::now();
        for stock in &self.0 {
            let query = query.clone();
            let transaction = transaction.clone();

            let path = format!("./buffer/submissions/CIK{}.json", stock.stock_id);
            trace!("reading file at path: \"{path}\"");
            let file: Sic = read_json(&path).await.expect("failed to read file");

            if set.contains(&stock.stock_id) {
                trace!("[{}] {} already inserted", &stock.ticker, &stock.title);
                continue;
            } else {
                set.insert(&stock.stock_id);
                let result = transaction
                    .execute(
                        &query,
                        &[
                            &stock.stock_id,
                            &stock.ticker,
                            &stock.title,
                            &file.sic_description,
                            &"US",
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => {
                        trace!("[{}] {} index data inserted", &stock.ticker, &stock.title)
                    }
                    Err(err) => {
                        error!(
                            "Failed to insert price data for [{}] {} | ERROR: {}",
                            &stock.ticker, &stock.title, err
                        );
                    }
                }
            }
        }

        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|e| {
                trace!("failed to commit pg_client transactions, {e}");
                e
            })?;

        debug!(
            "Stock index inserted - elapsed time: {}",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

/// Individual stock behaviour; i.e., each ticker in the list needs to process price & metrics
/// data (and any tertiary data) separately.
#[derive(Debug, Deserialize)]
pub struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    pub stock_id: String,
    pub ticker: String,
    pub title: String,
}

impl Ticker {
    /// Fetch & inser stock price dataset to PostgreSQL.
    pub async fn prices(
        &self,
        http_client: &reqwest::Client,
        pg_client: &mut tokio_postgres::Client,
    ) -> anyhow::Result<()> {
        // price data intervals
        for interval in &["1d", "1wk", "1mo", "3mo"] {
            // "max" keyword not supported as suggested
            let range = if *interval == "1m" { "8d" } else { "10y" };

            // fetch price dataset from Yahoo Finance
            let dataset = match prices::fetch(
                http_client,
                &self.ticker,
                &self.title,
                interval,
                range,
            )
            .await
            {
                Ok(data) => data,
                Err(e) => {
                    error!(
                        "Failed to fetch price data for [{}] {}: {}",
                        &self.ticker, &self.title, e
                    );
                    vec![]
                }
            };

            let query = pg_client.prepare("
                INSERT INTO stock.prices (stock_id, time, interval, opening, high, low, closing, adj_close, volume)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (stock_id, time, interval) DO NOTHING"
            ).await?;

            let transaction = Arc::new(pg_client.transaction().await?);

            // load to `stock.prices`
            let time = std::time::Instant::now();

            let mut stream = stream::iter(dataset);
            while let Some(cell) = stream.next().await {
                let query = query.clone();
                let transaction = transaction.clone();
                async move {
                    let result = transaction
                        .execute(
                            &query,
                            &[
                                &self.stock_id,
                                &cell.time,
                                &interval,
                                &cell.open,
                                &cell.high,
                                &cell.low,
                                &cell.close,
                                &cell.adj_close,
                                &cell.volume,
                            ],
                        )
                        .await;

                    match result {
                        Ok(_) => trace!("[{}] {} price data inserted", &self.ticker, &self.title),
                        Err(err) => error!(
                            "Failed to insert price data for [{}] {} | ERROR: {}",
                            &self.ticker, &self.title, err
                        ),
                    }
                }
                .await;
            }

            Arc::into_inner(transaction)
                .expect("failed to unpack Transaction from Arc")
                .commit()
                .await?;

            debug!(
                "[{}] {} priceset insert - elapsed time: {}",
                &self.ticker,
                &self.title,
                time.elapsed().as_millis()
            );
        }

        Ok(())
    }

    // Fetch & insert metrics dataset (given an unzipped file) to PostgreSQL.
    pub async fn metrics(&self, pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
        // load from local, unzipped file
        let metrics = match metrics::fetch(&self.stock_id, &self.ticker, &self.title).await {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to fetch metrics data for [{}] {}: {}",
                    self.ticker, self.title, e
                );
                vec![]
            }
        };

        let query = pg_client
            .prepare(
                "
            INSERT INTO stock.metrics (stock_id, dated, metric, val, unit, taxonomy)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (stock_id, dated, metric, val, unit, taxonomy) DO NOTHING
            ",
            )
            .await?;

        let transaction = Arc::new(pg_client.transaction().await?);

        let time = std::time::Instant::now();
        let mut stream = stream::iter(metrics);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                let result = transaction
                    .execute(
                        &query,
                        &[
                            &self.stock_id,
                            &cell.dated,
                            &cell.metric,
                            &cell.val,
                            &cell.unit,
                            &cell.taxonomy,
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => trace!("[{}] {} metric data inserted", self.ticker, self.title),
                    Err(err) => error!(
                        "Failed to insert metrics data for [{}] {} | ERROR: {}",
                        self.ticker, self.title, err
                    ),
                }
            }
            .await;
        }

        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await?;
        debug!(
            "[{}] {} metricset inserted, elapsed time: {}",
            self.ticker,
            self.title,
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// Custom Deserialize Tickers
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
