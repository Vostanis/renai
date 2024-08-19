use crate::endp::{sec, yahoo_finance as yf};
use serde::{Deserialize, Serialize};

/// Final output data collection for a single stock (e.g., Apple, Nvidia, Meta, etc.)
#[derive(Deserialize, Serialize, Debug)]
pub struct StockData {
    pub core: CoreSet, // (SEC)
    pub price: PriceSet, // (Yahoo! Finance)

                       // todo!
                       // ------------------------------
                       // pub patents: Patents, // (Google)
                       // pub holders: Holders, // (US gov - maybe finnhub)
                       // pub news: News, // (Google)
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
pub type CoreSet = Vec<sec::CoreCell>;

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
pub type PriceSet = Vec<yf::PriceCell>;
