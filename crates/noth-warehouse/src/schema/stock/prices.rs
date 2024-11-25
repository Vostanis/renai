#![allow(dead_code)]

use super::index::{Sec, Ticker, PRICE_QUERY};
use crate::api::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

///////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Prices from Yahoo Finance, per ticker
//
///////////////////////////////////////////////////////////////////////////////////////////////////////

impl Sec {
    pub async fn scrape_prices(
        ticker: Ticker,
        http_client: &HttpClient,
        pg_client: &mut PgClient,
    ) -> anyhow::Result<()> {
        let data = fetch(http_client, &ticker.ticker, &ticker.title, "1d", "10y").await?;
        Self::insert(data, pg_client, ticker).await?;
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

#[async_trait]
impl Postgres<Prices> for Sec {
    type Info = Ticker;

    async fn insert(
        data: Prices,
        pg_client: &mut PgClient,
        info: Self::Info,
    ) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&PRICE_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(data);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            let info = &info;
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &cell.stock_id,
                            &cell.time,
                            &cell.interval,
                            &cell.open,
                            &cell.high,
                            &cell.low,
                            &cell.close,
                            &cell.adj_close,
                            &cell.volume,
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("Prices inserted for [{}] {}", &info.ticker, &info.title),
                    Err(e) => {
                        error! {"Price insertion error for [{}] {}: {e}", &info.ticker, &info.title}
                    }
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
            "[{}] {} priceset inserted. Elapsed time: {} ms",
            &info.ticker,
            &info.title,
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// Time taken: 45-120 ms per ticker.

fn url(ticker: &str, interval: &str, range: &str) -> String {
    let tckr = ticker.to_uppercase();
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{tckr}?symbol={tckr}&interval={interval}&range={range}&events=div|split|capitalGains",
    )
}

// Forgone Http<Prices> because of birds-eye layout of the SEC process
pub(crate) async fn fetch(
    client: &Client,
    ticker: &String,
    title: &String,
    interval: &str,
    range: &str,
) -> anyhow::Result<Prices> {
    let price = {
        let url = url(&ticker, interval, range);
        trace!("Fetching price data for [{ticker}] {title} from Yahoo Finance");
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("[{ticker}] {title} price fetching error: {e}\nURL: {url}");
                e
            })?
            .bytes()
            .await
            .map_err(|e| {
                error!("[{ticker}] {title} byte trasnformation error: {e}\nURL: {url}");
                e
            })?;

        // error check the deserialization
        trace!("Deserializing price data for [{ticker}] {title} from Yahoo Finance");
        let de = match serde_json::from_slice::<PriceHistory>(&response) {
            Ok(data) => data,
            Err(e) => {
                error!("[{ticker}] {title} deserialization error: {e}\nURL: {url})");
                return Err(e.into());
            }
        };
        trace!("Price data fetched & deserialized successfully for [{ticker}] {title}");

        // scale Yahoo's data and transform it
        if let Some(data) = de.chart.result {
            trace!("Transforming price data for [{ticker}] {title}");
            let base = &data[0];
            let price = &base.indicators.quote[0];
            let adjclose = &base.indicators.adjclose[0].adjclose;
            let timestamps = &base.timestamp;
            price
                .open
                .iter()
                .zip(price.high.iter())
                .zip(price.low.iter())
                .zip(price.close.iter())
                .zip(price.volume.iter())
                .zip(adjclose.iter())
                .zip(timestamps.iter())
                .map(
                    |((((((open, high), low), close), volume), adj_close), timestamp)| PriceCell {
                        stock_id: ticker.clone(),
                        time: DateTime::from_timestamp(*timestamp, 0).expect("invalid timestamp"),
                        interval: interval.to_string(),
                        open: *open,
                        high: *high,
                        low: *low,
                        close: *close,
                        adj_close: *adj_close,
                        volume: *volume,
                    },
                )
                .collect::<Prices>()
        } else {
            error!("[{ticker}] {title} containted no \"chart.result\" object\nURL: {url}");
            vec![]
        }
    };

    Ok(price)
}

///////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Deserialization
//
///////////////////////////////////////////////////////////////////////////////////////////////////////

// Output: Price
type Prices = Vec<PriceCell>;

#[derive(Debug)]
pub struct PriceCell {
    pub stock_id: String,
    pub time: DateTime<Utc>,
    pub interval: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: i64,
}

// Input: Yahoo Finance
#[derive(Deserialize, Debug)]
pub struct PriceHistory {
    pub chart: PriceResponse,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PriceResponse {
    pub result: Option<Vec<PriceCategories>>,
}

#[derive(Deserialize, Debug)]
pub struct PriceCategories {
    pub timestamp: Vec<i64>,
    pub indicators: Indicators,
}

#[derive(Deserialize, Debug)]
pub struct Indicators {
    pub quote: Vec<Quote>,
    pub adjclose: Vec<AdjClose>,
}

#[derive(Deserialize, Debug)]
pub struct Quote {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<i64>,
}

#[derive(Deserialize, Debug)]
pub struct AdjClose {
    pub adjclose: Vec<f64>,
}
