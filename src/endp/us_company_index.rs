use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap as Map;
use std::env;

#[derive(Deserialize, Serialize, Debug)]
pub struct StockIndex {
    #[serde(deserialize_with = "de_cik")]
    pub cik_str: String,
    pub ticker: String,
    pub title: String,
}

// CIK code can either be a 10-digit string, or shortened number; de_cik handles both
pub fn de_cik<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        // if it's a num, pad it as a 10-char string
        serde_json::Value::Number(num) => {
            if let Some(i32_value) = num.as_i64() {
                // as_i64() does the same job for i32
                Ok(format!("{:010}", i32_value))
            } else {
                Err(serde::de::Error::custom(
                    "ERROR! Unable to parse i32 from JSON",
                ))
            }
        }

        // if it's a string, then Ok()
        serde_json::Value::String(s) => Ok(s),

        // else return an error (it can't be correct type)
        _ => Err(serde::de::Error::custom("ERROR! Invalid type for CIK")),
    }
}

pub async fn extran(client: &reqwest::Client) -> anyhow::Result<Vec<StockIndex>> {
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
