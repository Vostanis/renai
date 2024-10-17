use dotenv::var;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error};

// API Documentation:
//      >> "https://binance-docs.github.io/apidocs/spot/en/#introduction"
//      >> "https://binance-docs.github.io/apidocs/spot/en/#market-data-endpoints"
//
// [
//   [
//     1499040000000,      // Kline open time
//     "0.01634790",       // Open price
//     "0.80000000",       // High price
//     "0.01575800",       // Low price
//     "0.01577100",       // Close price
//     "148976.11427815",  // Volume
//     1499644799999,      // Kline Close time
//     "2434.19055334",    // Quote asset volume
//     308,                // Number of trades
//     "1756.87402397",    // Taker buy base asset volume
//     "28.46694368",      // Taker buy quote asset volume
//     "0"                 // Unused field, ignore.
//   ],
//   [
//      ...
//   ],
//   ...
// ]
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub(crate) struct Kline {
    pub(crate) timestamp: i64,
    pub(crate) opening: String,
    pub(crate) high: String,
    pub(crate) low: String,
    pub(crate) closing: String,
    pub(crate) volume: String,
    pub(crate) _close_timestamp: IgnoredAny,
    pub(crate) _quote_asset_volume: IgnoredAny,
    pub(crate) trades: i64,
    pub(crate) _taker_buy_base_asset_volume: IgnoredAny,
    pub(crate) _taker_buy_quote_asset_volume: IgnoredAny,
    pub(crate) _unused: IgnoredAny,
}

impl<'de> Visitor<'de> for Kline {
    type Value = Kline;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Array of Klines")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let packet = Kline {
            timestamp: seq.next_element::<i64>()?.expect("i64 timestamp"),
            opening: seq.next_element::<String>()?.expect("String open"),
            high: seq.next_element::<String>()?.expect("String high"),
            low: seq.next_element::<String>()?.expect("String low"),
            closing: seq.next_element::<String>()?.expect("String close"),
            volume: seq.next_element::<String>()?.expect("String volume"),
            _close_timestamp: seq
                .next_element::<IgnoredAny>()?
                .expect("i64 close timestamp"),
            _quote_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String quote asset volume"),
            trades: seq.next_element::<i64>()?.expect("i64 number of trades"),
            _taker_buy_base_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String taker buy base asset volume"),
            _taker_buy_quote_asset_volume: seq
                .next_element::<IgnoredAny>()?
                .expect("String taker buy quote asset volume"),
            _unused: seq
                .next_element::<IgnoredAny>()?
                .expect("String unused (ignore this field)"),
        };

        Ok(packet)
    }
}

fn guarantee_f64(value: Value) -> f64 {
    match value {
        Value::Number(num) => num.as_f64().unwrap(),
        Value::String(string) => string.parse::<f64>().unwrap(),
        _ => panic!("unexpected value"),
    }
}

pub(crate) struct Binance<'a> {
    client: Arc<reqwest::Client>,
    symbols: Vec<&'a str>,
}

use reqwest::header::{HeaderMap, HeaderValue};
#[allow(dead_code)]
impl<'a> Binance<'a> {
    fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            HeaderValue::from_str(&var("BINANCE_API").expect("BINANCE_API not found"))
                .expect("failed to set BINANCE_API as X-MBX-APIKEY header"),
        );

        Self {
            client: Arc::new(
                reqwest::ClientBuilder::new()
                    .default_headers(headers)
                    .build()
                    .unwrap(),
            ),
            symbols: vec![
                "BTCUSDT", "ETHUSDT", "SOLUSDT", "SUIUSDT", "KASUSDT", "ALPHUSDT", "ZENUSDT",
            ],
        }
    }

    pub(crate) async fn fetch(pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
        let api = Self::new();

        debug!("fetching binance crypto prices");

        let mut symbol_stream = stream::iter(&api.symbols);
        while let Some(&symbol) = symbol_stream.next().await {
            let time = std::time::Instant::now();
            let _ = async {
                debug!("fetching {}", symbol);
                let response: Vec<Kline> = api
                    .client
                    .get(format!(
                        "https://api.binance.com/api/v3/klines?symbol={}&interval=1d",
                        symbol
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;

                let query = pg_client
                    .prepare(
                        "
                INSERT INTO crypto.prices (pair, dated, opening, high, low, closing, volume, trades, source)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ",
                    )
                    .await?;

                let transaction = Arc::new(pg_client.transaction().await?);

                debug!("inserting binance priceset for {}", symbol);
                let mut stream = stream::iter(response);
                while let Some(cell) = stream.next().await {
                    let query = query.clone();
                    let transaction = transaction.clone();
                    async move {
                        let result = transaction
                            .execute(
                                &query,
                                &[
                                    &symbol,
                                    &ts_to_date(cell.timestamp), // <-- to_date()
                                    &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                                    &cell.high.parse::<f64>().expect("String -> f64 High"),
                                    &cell.low.parse::<f64>().expect("String -> f64 Low"), // all these values need String -> f64
                                    &cell.closing.parse::<f64>().expect("String -> f64 Closing"),
                                    &cell.volume.parse::<f64>().expect("String -> f64 Volume"), // String -> i64
                                    &cell.trades, // number of trades
                                    &"binance",
                                ],
                            )
                            .await;

                        match result {
                            Ok(_) => tracing::trace!("inserting binance price data for {}", symbol),
                            Err(err) => error!(
                                "Failed to insert price data for {} from Binance | ERROR: {}",
                                &symbol, err
                            ),
                        }
                    }
                    .await;
                }

                Arc::into_inner(transaction)
                    .expect("failed to unpack Transaction from Arc")
                    .commit()
                    .await?;
                debug!(
                    "Binance priceset inserted. Elapsed time: {}",
                    time.elapsed().as_millis()
                );

                Ok::<(), anyhow::Error>(())
            }
            .await;
        }

        Ok(())
    }
}

fn ts_to_date(timestamp: i64) -> chrono::NaiveDate {
    chrono::DateTime::from_timestamp_millis(timestamp)
        .expect("Expected Vector of Timestamp integers")
        .date_naive()
}
