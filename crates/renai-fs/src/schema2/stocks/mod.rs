/// Core metric data for companies; includes:
/// 1. US   [*source*]: ""
pub mod core;

/// The index table for stocks; a list of companies, in effect. Includes:
/// 1. US   [*source*]: "https://www.sec.gov/files/company_tickers.json"
pub mod index;

/// Price data for stock. Includes:
/// 1. Yahoo! Finance
pub mod price;

/// Common deserialization methods for Stocks.
pub mod common_de;