use base64::prelude::{Engine, BASE64_STANDARD};
use dotenv::var;
use hmac::{Hmac, Mac};
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error};

// RESPONSES
//
// Param 	Description
// time 	Start time of the candle cycle
// open 	Opening price
// close 	Closing price
// high 	Highest price
// low 	        Lowest price
// volume 	Transaction volume(One-sided transaction volume)
// turnover 	Transaction amount(One-sided transaction amount)
#[derive(Deserialize, Debug)]
struct KuCoinWrapper {
    data: Vec<Kline>,
}

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

// Signing Documentation: https://www.kucoin.com/docs/basic-info/connection-method/authentication/signing-a-message
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

pub(crate) struct KuCoin<'a> {
    client: Arc<reqwest::Client>,
    symbols: Vec<&'a str>,
}

use reqwest::header::{HeaderMap, HeaderValue};
#[allow(dead_code)]
impl<'a> KuCoin<'a> {
    fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "kc-api-key",
            HeaderValue::from_str(&var("KUCOIN_API").expect("kucoin_api not found"))
                .expect("failed to set kucoin_api as kc-api-key header"),
        );
        headers.insert(
            "kc-api-version",
            HeaderValue::from_str(&"2").expect("failed to set kc-api-version to \"2\""),
        );

        Self {
            client: Arc::new(
                reqwest::ClientBuilder::new()
                    .default_headers(headers)
                    .build()
                    .unwrap(),
            ),
            symbols: vec![
                "BTC-USDT",
                "ETH-USDT",
                "SOL-USDT",
                "SUI-USDT",
                "KAS-USDT",
                "ALPH-USDT",
                "ZEN-USDT",
            ],
        }
    }

    pub async fn fetch(pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
        let api = Self::new();

        let mut stream = stream::iter(&api.symbols);
        while let Some(symbol) = stream.next().await {
            // api parameters
            let url =
                format!("https://api.kucoin.com/api/v1/market/candles?type=1day&symbol={symbol}");
            let timestamp = timestamp();
            let passphrase = encrypt(var("KUCOIN_PRIVATE")?, var("KUCOIN_PASSPHRASE")?);
            let sign = sign(&url, var("KUCOIN_PRIVATE")?, timestamp.clone());

            // async parameters
            let client = api.client.clone();

            let time = std::time::Instant::now();

            // exe
            let _ = async {
                let klines: KuCoinWrapper = client
                    .get(url)
                    .header("KC-API-TIMESTAMP", timestamp)
                    .header("KC-API-PASSPHRASE", passphrase)
                    .header("KC-API-SIGN", sign)
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

                let mut stream = stream::iter(klines.data);
                while let Some(kline) = stream.next().await {
                    let query = query.clone();
                    let transaction = transaction.clone();

                    async move {
                        let result = transaction
                            .execute(
                                &query,
                                &[
                        &symbol.replace("-", ""),
                        &ts_to_date(kline.time.parse::<i64>().expect("String -> i64 Time")),
                        &kline.opening.parse::<f64>().expect("String -> f64 Opening"),
                        &kline.high.parse::<f64>().expect("String -> f64 Opening"),
                        &kline.low.parse::<f64>().expect("String -> f64 Opening"),
                        &kline.closing.parse::<f64>().expect("String -> f64 Opening"),
                        &kline.volume.parse::<f64>().expect("String -> f64 Opening"),
                        &None::<i64>,
                        &"kucoin",
                                ],
                            )
                            .await;

                        match result {
                            Ok(_) => (),
                            Err(e) => {
                                error!("error inserting kucoin data: {}", e);
                            }
                        }
                    }.await;
                };

                Arc::into_inner(transaction)
                    .expect("failed to unpack Transaction from Arc")
                    .commit()
                    .await?;
                debug!(
                    "kucoin priceset inserted. Elapsed time: {}",
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
    chrono::DateTime::from_timestamp(timestamp, 0)
        .expect("Expected Vector of Timestamp integers")
        .date_naive()
}
