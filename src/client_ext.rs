// use super::Datasets;
use crate::endp::yahoo_finance as yf;
use crate::www;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::Future;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CouchDocument {
    _id: String,
    _rev: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Document<T> {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _rev: String,
    pub data: T,
}

pub trait ClientExt {
    fn insert_doc<T>(
        &self,
        data: &T,
        conn: &str,
        doc_id: &str,
    ) -> impl Future<Output = Result<()>> + Send
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Sync;

    fn fetch_price_data(
        &self,
        ticker: &String,
        title: &String,
    ) -> impl Future<Output = Result<Vec<yf::PriceCell>>> + Send;

    fn fetch_price(
        &self,
        ticker: &String,
        title: &String,
    ) -> impl Future<Output = Result<Vec<yf::PriceCell>>> + Send;
}

impl ClientExt for Client {
    async fn insert_doc<T>(&self, data: &T, conn: &str, doc_id: &str) -> Result<()>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Sync,
    {
        // check if the document already exists with a GET request
        let conn = format!("{conn}/{doc_id}");
        let client = &self;
        let response = client
            .get(conn.clone())
            .send()
            .await
            .expect("failed to retrieve GET response");

        match response.status() {
            // "if the file already exists ..."
            reqwest::StatusCode::OK => {
                // retrieve current Revision ID
                let text = response
                    .text()
                    .await
                    .expect("failed to translate response to text");
                let current_file: CouchDocument = serde_json::from_str(&text)
                    .expect("failed to read current revision to serde schema");

                // PUT the file up with current Revision ID
                let mut updated_file = json!(data);
                updated_file["_rev"] = json!(current_file._rev);
                let _second_response = client
                    .put(conn)
                    .json(&updated_file)
                    .send()
                    .await
                    .expect("failed to retrieve PUT response");
            }

            // "if the file does not exist ..."
            reqwest::StatusCode::NOT_FOUND => {
                // PUT the new file up, requiring no Revision ID (where we use an empty string)
                let new_file = json!(data);
                let _second_response = client
                    .put(conn)
                    .json(&new_file)
                    .send()
                    .await
                    .expect("failed to retrieve PUT response");
            }

            _ => eprintln!("Unexpected status code found - expected `OK` or `NOT_FOUND`"),
        }
        Ok(())
    }

    async fn fetch_price(&self, ticker: &String, title: &String) -> Result<Vec<yf::PriceCell>> {
        let price_url = www::price_url(ticker).await;
        let price = {
            let price_response: yf::PriceHistory = self.get(price_url).send().await?.json().await?;

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
                            |((((((open, high), low), close), volume), adj_close), date)| {
                                yf::PriceCell {
                                    dated: date.clone(),
                                    open: *open,
                                    high: *high,
                                    low: *low,
                                    close: *close,
                                    adj_close: *adj_close,
                                    volume: *volume,
                                }
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

    async fn fetch_price_data(
        &self,
        ticker: &String,
        title: &String,
    ) -> Result<Vec<yf::PriceCell>> {
        let out = yf::extran(&self, www::price_url(ticker).await, ticker, title).await?;
        Ok(out)
    }
}
