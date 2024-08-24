use super::core::us::CoreCell;
use super::price::yahoo_finance::PriceCell;
use serde::{Deserialize, Serialize};

/// Final output data collection for a single stock (e.g., Apple, Nvidia, Meta, etc.)
#[derive(Deserialize, Serialize, Debug)]
pub struct StockDataset {
    pub core: CoreSet,
    pub price: PriceSet,

                       // todo!
                       // ------------------------------
                       // pub patents: Patents, // (Google)
                       // pub holders: Holders, // (US gov - maybe finnhub)
                       // pub news: News, // (Google)
}

impl StockDataset {
    async fn _collect(_ticker: &str) {
        todo!()
    }
}

/// Core data collection (e.g., Revenue, EPS, Debt, etc.)
/// ```rust
/// "core": [
///      {
///          "dated": "2021-01-01",
///          "Revenue": 1298973.0,
///          "DilutedEPS": 2.7,
///      },
///      {
///          "dated": "2022-01-01",
///          "Revenue": 23112515.0,
///          "DilutedEPS": 1.72,
///      },
///      // ...
/// ]
/// ```
pub type CoreSet = Vec<CoreCell>;

/// Price data collection (i.e., Open, High, Low, Close, Adj. Close)
/// ```rust
/// "price": [
///      {
///          "dated": "2021-01-01",
///          "open": 123.0,
///          "adj_close": 124.2,
///      },
///      {
///          "dated": "2022-01-01",
///          "open": 124.2,
///          "adj_close": 122.0,
///      },
///      // ...
/// ]
/// ```
pub type PriceSet = Vec<PriceCell>;