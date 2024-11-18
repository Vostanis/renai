use crate::api::{Api, Http, Postgres};
use crate::schema::crypto::index::{PAIRS, PRICE_QUERY};
use async_trait::async_trait;
use dotenv::var;
use reqwest::header::HeaderValue;
use reqwest::Client as HttpClient;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_postgres::Client as PgClient;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

/////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Documentation:
//      - https://binance-docs.github.io/apidocs/spot/en/#introduction
//      - https://binance-docs.github.io/apidocs/spot/en/#market-data-endpoints
//
pub struct Binance;

impl Binance {
    fn build_client() -> HttpClient {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            HeaderValue::from_str(&var("BINANCE_API").expect("BINANCE_API not found"))
                .expect("failed to set BINANCE_API as X-MBX-APIKEY header"),
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("Binance Client to build");
        client
    }

    pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let http_client = Self::build_client();
        Self::etl(&http_client, pg_client).await?;
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Api<Klines> for Binance {
    async fn etl(http_client: &HttpClient, pg_client: &mut PgClient) -> anyhow::Result<()> {
        for (id, symbol) in PAIRS.clone() {
            let url = format!("https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1d");
            let data = Self::fetch(http_client, &url).await?;
            Self::insert(data, pg_client, (id, symbol.to_string())).await?;
        }
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Http<Klines> for Binance {
    async fn fetch(client: &HttpClient, url: &String) -> anyhow::Result<Klines> {
        let data = Self::fetch_de(client, url).await?;
        Ok(data)
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Postgres<Klines> for Binance {
    type Info = (i32, String);

    async fn insert(
        data: Klines,
        pg_client: &mut PgClient,
        additional_info: Self::Info,
    ) -> anyhow::Result<()> {
        // start the clock
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&PRICE_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);
        let (id, symbol) = additional_info;

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(data);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            let symbol = &symbol;
            async move {
                let result = transaction
                    .execute(
                        &query,
                        &[
                            &id,
                            &"1d",
                            &cell.timestamp,
                            &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.high.parse::<f64>().expect("String -> f64 High"),
                            &cell.low.parse::<f64>().expect("String -> f64 Low"), // all these values need String -> f64
                            &cell.closing.parse::<f64>().expect("String -> f64 Closing"),
                            &cell.volume.parse::<f64>().expect("String -> f64 Volume"), // String -> i64
                            &cell.trades, // number of trades
                            &"Binance",
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => trace!("inserting Binance price data for {}", &symbol),
                    Err(err) => error!(
                        "Failed to insert price data for {} from Binance | ERROR: {}",
                        &symbol, err
                    ),
                }
            }
            .await;
        }

        // unpack the transcation and commit it to the database
        Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
            .map_err(|e| {
                error!("failed to commit transaction for {symbol} from Binance");
                e
            })?;

        debug!(
            "Binance priceset inserted. Elapsed time: {} ms",
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
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
//     "1756.87402397",    // Taker buy base asset volum.e
//     "28.46694368",      // Taker buy quote asset volume
//     "0"                 // Unused field, ignore.
//   ],
//   [
//      ...
//   ],
//   ...
// ]
type Klines = Vec<Kline>;

#[derive(Deserialize, Debug)]
struct Kline {
    timestamp: i64,
    opening: String,
    high: String,
    low: String,
    closing: String,
    volume: String,
    _close_timestamp: IgnoredAny,
    _quote_asset_volume: IgnoredAny,
    trades: i64,
    _taker_buy_base_asset_volume: IgnoredAny,
    _taker_buy_quote_asset_volume: IgnoredAny,
    _unused: IgnoredAny,
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
        Ok(Kline {
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
        })
    }
}
