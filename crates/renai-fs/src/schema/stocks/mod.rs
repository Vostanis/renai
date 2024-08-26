/// Core metric data for companies; includes:
/// 1. US   [*source*]: "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip"
pub mod core;

/// Output for Stocks.
pub mod exe;

/// The index table for stocks; a list of companies, in effect. Includes:
/// 1. US   [*source*]: "https://www.sec.gov/files/company_tickers.json"
pub mod index;

/// Price data for stock. Includes:
/// 1. Yahoo! Finance
pub mod price;