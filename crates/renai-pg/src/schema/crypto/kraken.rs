use crate::schema::crypto::index::PAIRS;
use dotenv::var;
use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashMap as Map;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error};

// RESPONSES
//
// "result": {
//      "BTCUSDT": [
//          [
//              1688671200,
//              "30306.1",
//              "30306.2",
//              "30305.7",
//              "30305.7",
//              "30306.1",
//              "3.39243896",
//              23
//          ],
//          [
//              ...
//          ],
//          ...
//      ],
//      "last": 15123121293,
//  }
#[derive(Deserialize, Debug)]
struct KrakenResponse {
    result: KrakenResult,
}

#[derive(Deserialize, Debug)]
struct KrakenResult(Map<String, Vec<Kline>>);

impl<'de> Visitor<'de> for KrakenResult {
    type Value = KrakenResult;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Map of Klines")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut result = Map::new();

        while let Some(klines) = map.next_entry::<String, Vec<Kline>>()? {
            if klines.0.as_str() != "last" {
                result.insert(klines.0, klines.1).unwrap();
            }
        }

        // while let Some(key) = map.next_key::<String>()? {
        //     if key.as_str() != "last" {
        //         result.insert(key, map.get(key));
        //     }
        // }
        Ok(KrakenResult(result))
    }
}

#[derive(Deserialize, Debug)]
struct Kline {
    time: i64,
    opening: String,
    high: String,
    low: String,
    closing: String,
    _vwap: IgnoredAny,
    volume: String,
    trades: i64,
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
            high: seq.next_element()?.expect("String high"),
            low: seq.next_element()?.expect("String low"),
            closing: seq.next_element()?.expect("String closing"),
            _vwap: seq.next_element()?.expect("vwap to ignore"),
            volume: seq.next_element()?.expect("String volume"),
            trades: seq.next_element()?.expect("i64 count"),
        })
    }
}

pub(crate) struct Kraken {
    client: Arc<reqwest::Client>,
    // symbols: Vec<&'a str>,
}

use reqwest::header::{HeaderMap, HeaderValue};
#[allow(dead_code)]
impl Kraken {
    fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "API-KEY",
            HeaderValue::from_str(&var("KRAKEN_API").expect("kraken_api not found"))
                .expect("failed to set kraken_api"),
        );

        Self {
            client: Arc::new(
                reqwest::ClientBuilder::new()
                    .default_headers(headers)
                    .build()
                    .unwrap(),
            ),
            // symbols: vec![
            //     "BTCUSDT", "ETHUSDT", "SOLUSDT", "SUIUSDT", "KASUSDT", "ALPHUSDT", "ZENUSDT",
            // ],
        }
    }

    pub async fn fetch(pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
        let api = Self::new();

        let pairs = PAIRS.clone();
        let mut stream = stream::iter(&pairs);
        while let Some(symbol) = stream.next().await {
            // api parameters
            let url = format!(
                "https://api.kraken.com/0/public/OHLC?pair={}&interval=1440",
                symbol.1
            );

            // async parameters
            let client = api.client.clone();

            let time = std::time::Instant::now();

            // exe
            let _ = async {
                let klines: KrakenResponse = client
                    .get(url)
                    .send()
                    .await
                    .expect("asdlasdlkasd")
                    .json()
                    .await
                    .expect("..sadlasd");
                println!("{:?}", klines);

                //             let query = pg_client
                //                 .prepare(
                //                     "
                //             INSERT INTO crypto.prices (pair, dated, opening, high, low, closing, volume, trades, source)
                //             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                //             ",
                //                 )
                //                 .await?;
                //
                //             let transaction = Arc::new(pg_client.transaction().await?);
                //
                //             let set = klines.result.get(&symbol.to_string()).expect("dataset of Klines");
                //             let mut stream = stream::iter(set);
                //             while let Some(kline) = stream.next().await {
                //                 let query = query.clone();
                //                 let transaction = transaction.clone();
                //
                //                 async move {
                //                     let result = transaction
                //                         .execute(
                //                             &query,
                //                             &[
                //                     &symbol.replace("-", ""),
                //                     &ts_to_date(kline.time),
                //                     &kline.opening.parse::<f64>().expect("String -> f64 Opening"),
                //                     &kline.high.parse::<f64>().expect("String -> f64 Opening"),
                //                     &kline.low.parse::<f64>().expect("String -> f64 Opening"),
                //                     &kline.closing.parse::<f64>().expect("String -> f64 Opening"),
                //                     &kline.volume.parse::<f64>().expect("String -> f64 Opening"),
                //                     &kline.trades,
                //                     &"kraken",
                //                             ],
                //                         )
                //                         .await;
                //
                //                     match result {
                //                         Ok(_) => (),
                //                         Err(e) => {
                //                             error!("error inserting kucoin data: {}", e);
                //                         }
                //                     }
                //                 }.await;
                //             };
                //
                //             Arc::into_inner(transaction)
                //                 .expect("failed to unpack Transaction from Arc")
                //                 .commit()
                //                 .await?;
                //             debug!(
                //                 "kucoin priceset inserted. Elapsed time: {}",
                //                 time.elapsed().as_millis()
                //             );

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
