#![allow(dead_code)]

use crate::schema::common_de::de_cik;
use crate::schema::stock::price::{url, Prices};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};

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

impl Tickers {
    // load Tickers to pg db
    // pub async fn load() -> anyhow::Result<()> {}

    // fetch prices and load to pg db
    // pub async fn prices() -> anyhow::Result<()> {}

    // fetch fundamentals and load to pg db
    // pub async fn fundamentals() -> anyhow::Result<()> {}
}

#[derive(Debug, Deserialize)]
pub(crate) struct Ticker {
    #[serde(rename = "cik_str", deserialize_with = "de_cik")]
    pub(crate) stock_id: String,
    pub(crate) ticker: String,
    pub(crate) title: String,
}

impl Ticker {
    // fetch stock price dataset
    pub async fn prices(&self, client: &reqwest::Client) -> anyhow::Result<Prices> {
        let url = url(&self.ticker);
        let json: Prices = client.get(url).send().await?.json().await?;
        Ok(json.0)
    }

    // fetch fundamentals dataset (given an unzipped file)
    // pub async fn fundamentals() -> anyhow::Result<()> {}
}

// Deserialize Tickers
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
        let mut tickers: Vec<Ticker> = Vec::new();

        // each entry is in the form of:
        // `0: { "cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc." },`
        while let Some((_, ticker)) = map.next_entry::<u16, Ticker>()? {
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
