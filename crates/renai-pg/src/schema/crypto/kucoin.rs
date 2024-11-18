use crate::api::{Api, Http, Postgres};
use crate::schema::crypto::index::{PAIRS, PRICE_QUERY};
use async_trait::async_trait;
use base64::prelude::{Engine, BASE64_STANDARD};
use dotenv::var;
use hmac::{Hmac, Mac};
use reqwest::header::HeaderValue;
use reqwest::Client as HttpClient;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;
use tokio_postgres::Client as PgClient;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct KuCoin;

impl KuCoin {
    fn build_client() -> HttpClient {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "KC-API-KEY",
            HeaderValue::from_str(&var("KUCOIN_API").expect("KUCOIN_API not found"))
                .expect("failed to set KUCOIN_API as X-MBX-APIKEY header"),
        );
        headers.insert(
            "KC-API-VERSION",
            HeaderValue::from_str(&"2").expect("failed to set kc-api-version to \"2\""),
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("KuCoin Client to build");
        client
    }

    pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let http_client = Self::build_client();
        Self::etl(&http_client, pg_client).await?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Api<Klines> for KuCoin {
    async fn etl(http_client: &HttpClient, pg_client: &mut PgClient) -> anyhow::Result<()> {
        // for interval in intervals {
        for (id, symbol) in PAIRS.clone() {
            let url =
                format!("https://api.kucoin.com/api/v1/market/candles?type=1day&symbol={symbol}");
            let data = Self::fetch(http_client, &url).await?;
            Self::insert(data, pg_client, (id, symbol.to_string())).await?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Http<Klines> for KuCoin {
    async fn fetch(client: &HttpClient, url: &String) -> anyhow::Result<Klines> {
        let timestamp = timestamp();
        let passphrase = encrypt(var("KUCOIN_PRIVATE")?, var("KUCOIN_PASSPHRASE")?);
        let sign = sign(&url, var("KUCOIN_PRIVATE")?, timestamp.clone());
        let data: Klines = client
            .get(url)
            .header("KC-API-TIMESTAMP", timestamp)
            .header("KC-API-PASSPHRASE", passphrase)
            .header("KC-API-SIGN", sign)
            .send()
            .await?
            .json()
            .await?;
        Ok(data)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Postgres<Klines> for KuCoin {
    type Info = (i32, String);

    async fn insert(
        data: Klines,
        pg_client: &mut PgClient,
        additional_info: Self::Info,
    ) -> anyhow::Result<()> {
        let time = ::std::time::Instant::now();

        let query = pg_client.prepare(&PRICE_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);
        let (id, symbol) = additional_info;

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
                            &cell.time.parse::<i64>().expect("String -> i64 Time"),
                            &cell.opening.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.high.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.low.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.closing.parse::<f64>().expect("String -> f64 Opening"),
                            &cell.volume.parse::<f64>().expect("String -> f64 Opening"),
                            &None::<i64>,
                            &"KuCoin",
                        ],
                    )
                    .await;

                match result {
                    Ok(_) => trace!("inserting KuCoin price data for {}", &symbol),
                    Err(err) => error!(
                        "Failed to insert price data for {} from KuCoin | ERROR: {}",
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

////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Security protocols
// Signing Documentation:
//      - https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message
//
fn encrypt(secret: String, input: String) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&secret.as_bytes()).unwrap();
    mac.update(input.as_bytes());
    let result = mac.finalize().into_bytes();
    let b64 = BASE64_STANDARD.encode(&result);
    b64
}

fn sign(url: &String, secret: String, timestamp: String) -> String {
    let url = url.replace("https://api.kucoin.com", "");
    let input = format!("{}{}{}", timestamp, "GET", url);
    encrypt(secret, input)
}

fn timestamp() -> String {
    chrono::Utc::now().timestamp_millis().to_string()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// All elements of the array are Strings
//
// [
//   [
//      time 	        // Start time of the candle cycle
//      open 	        // Opening price
//      close 	        // Closing price
//      high 	        // Highest price
//      low 	        // Lowest price
//      volume 	        // Transaction volume(One-sided transaction volume)
//      turnover 	// Transaction amount(One-sided transaction amount)
//  ],
//  [
//      ...
//  ],
//  ...
// ]
type Klines = Vec<Kline>;

#[derive(Deserialize, Debug)]
struct Kline {
    time: String,
    opening: String,
    closing: String,
    high: String,
    low: String,
    volume: String,
    _turnover: IgnoredAny,
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
            time: seq.next_element()?.expect("String timestamp"),
            opening: seq.next_element()?.expect("String opening"),
            closing: seq.next_element()?.expect("String closing"),
            high: seq.next_element()?.expect("String high"),
            low: seq.next_element()?.expect("String low"),
            volume: seq.next_element()?.expect("String volume"),
            _turnover: seq.next_element()?.expect("turnover to ignore"),
        })
    }
}
