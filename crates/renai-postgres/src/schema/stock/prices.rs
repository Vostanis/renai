#![allow(dead_code)]

use crate::schema::common::convert_timestamp;
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
) -> anyhow::Result<Vec<PriceCell>> {
    let price = {
        let url = url(&ticker).await;
        let response: PriceHistory = client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                error!("[{ticker}] {title} failed to fetch Price response | ERROR: {e} | URL: {url}");
                e
            })?
            .json()
            .await
            .map_err(|e| {
                error!(
                    "[{ticker}] {title} failed to transform Price response | ERROR: {e} | URL: {url}"
                );
                e
            })?;

        if let Some(data) = response.chart.result {
            trace!("Price data fetched successfully for [{}] {}", ticker, title);
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
                        dated: convert_timestamp(*timestamp),
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
            error!("[{ticker}] {title} failed to fetch Price data | URL: {url}");
            vec![]
        }
    };

    Ok(price)
}

pub(crate) async fn url(ticker: &str) -> String {
    let tckr = ticker.to_uppercase();
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{tckr}?symbol={tckr}&interval={}&range={}&events=div|split|capitalGains",
        "1d", // intervals
        "3y" // range
    )
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// Output
// ==========================================================================
#[derive(Debug)]
pub struct PriceCell {
    pub stock_id: String,
    pub dated: chrono::NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: i32,
}

// >> Input: Yahoo Finance
// ==========================================================================
#[derive(Deserialize, Debug)]
pub struct PriceHistory {
    pub chart: PriceResponse,
}

#[derive(Deserialize, Debug)]
pub struct PriceResponse {
    pub result: Option<Vec<PriceCategories>>,
}

#[derive(Deserialize, Debug)]
pub struct PriceCategories {
    pub timestamp: Vec<u32>,
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
    pub volume: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct AdjClose {
    pub adjclose: Vec<f64>,
}
