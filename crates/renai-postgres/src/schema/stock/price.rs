#![allow(dead_code)]

use crate::schema::common_de::de_timestamp_to_naive_date;
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn fetch(
    client: &Client,
    ticker: &String,
    title: &String,
) -> anyhow::Result<Vec<PriceCell>> {
    let price = {
        let url = url(&ticker).await;
        let response: PriceHistory = client.get(url).send().await?.json().await?;
        match response.chart.result {
            Some(data) => {
                let base = &data[0];
                let price = &base.indicators.quote[0];
                let adjclose = &base.indicators.adjclose[0].adjclose;
                let dates = &base.dates;
                price
                    .open
                    .iter()
                    .zip(price.high.iter())
                    .zip(price.low.iter())
                    .zip(price.close.iter())
                    .zip(price.volume.iter())
                    .zip(adjclose.iter())
                    .zip(dates.iter())
                    .map(
                        |((((((open, high), low), close), volume), adj_close), date)| PriceCell {
                            stock_id: ticker.clone(),
                            dated: *date,
                            open: *open,
                            high: *high,
                            low: *low,
                            close: *close,
                            adj_close: *adj_close,
                            volume: *volume,
                        },
                    )
                    .collect::<Vec<_>>()
            }

            None => {
                log::warn!("[{ticker}] {title} failed to extract Price data; filling with an empty array instead");
                vec![] // return an empty vec in the absence of actual dataset
            }
        }
    };
    Ok(price)
}

pub(crate) async fn url(ticker: &str) -> String {
    let tckr = ticker.to_uppercase();
    format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{tckr}?symbol={tckr}&interval={}&range={}&events=div|split|capitalGains",
        "1d",
        "3y"
    )
}

#[derive(Deserialize, Debug)]
pub(crate) struct PriceCell {
    pub(crate) stock_id: String,
    pub(crate) dated: NaiveDate,
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
    #[serde(rename = "timestamp", deserialize_with = "de_timestamp_to_naive_date")]
    pub dates: Vec<NaiveDate>,
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
