#![allow(dead_code)]

use reqwest::Client;
use serde::Deserialize;

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
                tracing::error!("[{ticker}] {title} failed to fetch Price response | ERROR: {e} | URL: {url}");
                e
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!(
                    "[{ticker}] {title} failed to transform Price response | ERROR: {e} | URL: {url}"
                );
                e
            })?;

        if let Some(data) = response.chart.result {
            tracing::trace!("Price data fetched successfully for [{}] {}", ticker, title);
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
                        dated: ts_to_date(*timestamp),
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
            tracing::error!("[{ticker}] {title} failed to fetch Price data | URL: {url}");
            vec![]
        }
    };
    Ok(price)
}

pub fn ts_to_date(timestamp: u32) -> chrono::NaiveDate {
    chrono::DateTime::from_timestamp(timestamp.into(), 0)
        .expect("Expected Vector of Timestamp integers")
        .date_naive()
}

pub(crate) async fn url(ticker: &str) -> String {
    let tckr = ticker.to_uppercase();
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{tckr}?symbol={tckr}&interval={}&range={}&events=div|split|capitalGains",
        "1d", // intervals
        "3y" // range
    )
}

#[derive(Debug)]
pub(crate) struct PriceCell {
    pub(crate) stock_id: String,
    pub(crate) dated: chrono::NaiveDate,
    pub(crate) open: f64,
    pub(crate) high: f64,
    pub(crate) low: f64,
    pub(crate) close: f64,
    pub(crate) adj_close: f64,
    pub(crate) volume: i32,
}

// `price` schema
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
