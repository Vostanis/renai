#![allow(dead_code)]

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use tracing::{error, trace};

/// Fetches prices data from Yahoo Finance, per ticker.
///
/// Time taken: 45-75 ms per ticker.
pub(crate) async fn fetch(
    client: &Client,
    ticker: &String,
    title: &String,
    interval: &str,
    range: &str,
) -> anyhow::Result<Vec<PriceCell>> {
    let price = {
        let url = url(&ticker, interval, range).await;
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
                // let response =
                //     serde_json::from_slice::<serde_json::Value>(&response).map_err(|e| {
                //         error!("could not derive response to serde_json::Value: {e}\nURL: {url}");
                //         e
                //     })?;
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
                .collect::<Vec<PriceCell>>()
        } else {
            error!("[{ticker}] {title} containted no \"chart.result\" object\nURL: {url}");
            vec![]
        }
    };

    Ok(price)
}

pub(crate) async fn url(ticker: &str, interval: &str, range: &str) -> String {
    let tckr = ticker.to_uppercase();
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{tckr}?symbol={tckr}&interval={interval}&range={range}&events=div|split|capitalGains",
    )
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Output
// ==========================================================================
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

// >> Input: Yahoo Finance
// ==========================================================================
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
