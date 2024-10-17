#![allow(dead_code)]

use crate::schema::common_de::de_cik;
use crate::schema::stock::metrics;
use crate::schema::stock::prices::{self};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////
// API Documentation: https://www.sec.gov/search-filings/edgar-application-programming-interfaces
kvapi::api! {
    name: Sec
    base: "https://www.sec.gov/files/"
    head: { "User-Agent": &dotenv::var("USER_AGENT")? }
    dict: { "company_tickers.json" -> Tickers }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// US Stock Tickers/Titles
#[derive(Debug)]
pub(crate) struct Tickers(pub(crate) Vec<Ticker>);

#[derive(Debug, Deserialize)]
pub(crate) struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    pub(crate) stock_id: String,
    pub(crate) ticker: String,
    pub(crate) title: String,
}

impl Ticker {
    // fetch stock price dataset
    pub async fn prices(
        &self,
        http_client: &reqwest::Client,
        pg_client: &mut tokio_postgres::Client,
    ) -> anyhow::Result<()> {
        // fetch price dataset from Yahoo Finance
        let dataset = prices::fetch(http_client, &self.ticker, &self.title).await?;

        let query = pg_client.prepare("
            INSERT INTO stock.prices (stock_id, dated, opening, high, low, closing, adj_close, volume)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
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
                            &cell.dated,
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
                    Ok(_) => {}
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

        Ok(())
    }

    // fetch fundamentals dataset (given an unzipped file)
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
            INSERT INTO stock.metrics (stock_id, dated, metric, val)
            VALUES ($1, $2, $3, $4)",
            )
            .await?;

        let transaction = Arc::new(pg_client.transaction().await?);

        // load to `stock.prices`
        let time = std::time::Instant::now();

        // load to `stock.metrics`
        let mut stream = stream::iter(metrics);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            async move {
                let result = transaction
                    .execute(
                        &query,
                        &[&self.stock_id, &cell.dated, &cell.metric, &cell.val],
                    )
                    .await;

                match result {
                    Ok(_) => {}
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
