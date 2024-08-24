use super::de_timestamps_to_naive_date;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn fetch(
    client: &Client,
    price_url: String,
    ticker: &String,
    title: &String,
) -> Result<Vec<PriceCell>> {
    let price = {
        let price_response: PriceHistory = client.get(price_url).send().await?.json().await?;

        match price_response.chart.result {
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
                            dated: date.clone(),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceCell {
    pub dated: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: i32,
}

// `price` schema
#[derive(Deserialize, Serialize, Debug)]
pub struct PriceHistory {
    pub chart: PriceResponse,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceResponse {
    pub result: Option<Vec<PriceCategories>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PriceCategories {
    #[serde(rename = "timestamp", deserialize_with = "de_timestamps_to_naive_date")]
    pub dates: Vec<String>,
    pub indicators: Indicators,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Indicators {
    pub quote: Vec<Quote>,
    pub adjclose: Vec<AdjClose>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Quote {
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AdjClose {
    pub adjclose: Vec<f64>,
}