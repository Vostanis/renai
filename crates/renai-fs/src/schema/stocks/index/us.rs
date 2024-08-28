use super::de_cik;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use std::env;

pub async fn fetch(client: &reqwest::Client) -> anyhow::Result<Vec<StockIndex>> {
    let user_agent =
        env::var("USER_AGENT").expect("Unable to find environment variable: USER_AGENT");
    let tickers_query = "https://www.sec.gov/files/company_tickers.json";
    let tickers_response: Map<u32, StockIndex> = client
        .get(tickers_query)
        .header("User-Agent", user_agent)
        .send()
        .await?
        .json()
        .await?;

    let tickers = tickers_response
        .values()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|row| StockIndex {
            cik_str: row.cik_str.clone(),
            ticker: row.ticker.clone(),
            title: row.title.clone(),
        })
        .collect::<Vec<_>>();

    Ok(tickers)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StockIndex {
    #[serde(deserialize_with = "de_cik")]
    pub cik_str: String,
    pub ticker: String,
    pub title: String,
}
