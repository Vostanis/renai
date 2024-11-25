use super::index::{Sec, Ticker, METRIC_QUERY};
use crate::api::*;
use crate::schema::common::convert_date_type;
use async_trait::async_trait;
use noth_util::read_json;
use serde::Deserialize;
use std::collections::HashMap as Map;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, info, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////
//
// US Stock Metrcics from the SEC's `companyfacts.zip`
//
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Sec {
    pub async fn scrape_metrics(ticker: Ticker, pg_client: &mut PgClient) -> anyhow::Result<()> {
        let data = fetch(&ticker).await?;
        Self::insert(data, pg_client, ticker).await?;
        Ok(())
    }
}

#[async_trait]
impl Postgres<Metrics> for Sec {
    type Info = Ticker;

    async fn insert(
        data: Metrics,
        pg_client: &mut PgClient,
        info: Self::Info,
    ) -> anyhow::Result<()> {
        let time = std::time::Instant::now();

        // preprocess pg query as transaction
        let query = pg_client.prepare(&METRIC_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        // iterate over the data stream and execute pg rows
        let mut stream = stream::iter(data);
        while let Some(cell) = stream.next().await {
            let query = query.clone();
            let transaction = transaction.clone();
            let info = &info;
            async move {
                match transaction
                    .execute(
                        &query,
                        &[
                            &info.stock_id,
                            &cell.dated,
                            &cell.metric,
                            &cell.val,
                            &cell.unit,
                            &cell.taxonomy,
                        ],
                    )
                    .await
                {
                    Ok(_) => trace!("Metrics inserted for [{}] {}", &info.ticker, &info.title),
                    Err(e) => error!(
                        "Metric insertion failure for [{}] {}: {e}",
                        &info.ticker, &info.title
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
            "[{}] {} metricset inserted, elapsed time: {}",
            &info.ticker,
            &info.title,
            time.elapsed().as_millis()
        );

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------
// Time taken: ~4-7s per ticker.

pub(crate) async fn fetch(ticker: &Ticker) -> anyhow::Result<Metrics> {
    let (cik_str, ticker, title) = (&ticker.stock_id, &ticker.ticker, &ticker.title);

    // construct the path
    let path = format!("./buffer/companyfacts/CIK{cik_str}.json");

    // read the file
    trace!("reading file at path: \"{path}\"");
    let json: Facts = read_json(&path).await.map_err(|e| {
        error!("failed to read file at \"{path}\" for [{ticker}] {title}: {e}");
        e
    })?;

    // read the JSON
    trace!("reformatting facts dataset for [{ticker}] {title}");
    let mut output: Vec<Metric> = vec![];
    for (dei_or_us_gaap, metric) in json.facts {
        match dei_or_us_gaap.as_str() {
            "dei" | "us-gaap" | "srt" | "invest" => {
                for (metric_name, dataset) in metric {
                    for (units, values) in dataset.units {
                        for cell in values {
                            output.push(Metric {
                                dated: convert_date_type(&cell.dated)?,
                                metric: metric_name.clone(),
                                val: cell.val,
                                unit: units.clone(),
                                taxonomy: dei_or_us_gaap.clone(),
                            });
                        }
                    }
                }
            }
            _ => info!("Unexpected dataset found in Company Fact data {dei_or_us_gaap}"),
        };
    }

    Ok(output)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//
// Deserialization
//
////////////////////////////////////////////////////////////////////////////////////////////////////

// Output

// [
//      {
//          "dated": "2021-01-01",      # types | rust: NaiveDate -> postgres: DATE
//          "metric": "Revenues",
//          "val": "213123123123"
//      },
//      ...
// ]
type Metrics = Vec<Metric>;

#[derive(Debug)]
pub struct Metric {
    pub dated: chrono::NaiveDate,
    pub metric: String,
    pub val: f64,
    pub unit: String,
    pub taxonomy: String,
}

// -------------------------------------------------------------------------------------------------
// Input

// {
//    "facts": {
#[derive(Deserialize, Debug)]
pub(crate) struct Facts {
    //                      vvvv == "MetricName"
    facts: Map<String, Map<String, MetricData>>,
    //          ^^^^  == "dei" or "us-gaap"
}

//          "dei": {
//              EntityCommonStockSharesOutstanding": {
#[derive(Deserialize, Debug)]
pub(crate) struct MetricData {
    units: Map<String, Vec<DataCell>>,
    //          ^^^^ == "shares" or "USD"
}
//                  "label":"Entity Common Stock, Shares Outstanding",
//                  "description":"Indicate number of shares or ...",
//                  "units": {

//                      "shares": [  <-- or "USD"

#[derive(Deserialize, Debug)]
pub(crate) struct DataCell {
    #[serde(rename = "end")]
    //                ^^^ "end" is a keyword in PostgreSQL, so it's renamed to "dated"
    dated: String,
    val: f64,
}
//                          {
//                              "end":"2009-06-30",
//                              "val":1545912443,
//                              "accn":"0001104659-09-048013",
//                              "fy":2009,
//                              "fp":"Q2",
//                              "form":"10-Q",
//                              "filed":"2009-08-07",
//                              "frame":"CY2009Q2I"
//                          },
//                          ...
//                      ]
//                  },
//              },
//              ...
//          },
//          "us-gaap": {
//               "label": "Accrued Income Taxes, Current",
//               "description": "Carrying amount as of the balance sheet ...",
//                  "units": {
//                      "USD": [
//                          {
//                              "end": "2007-12-31",
//                              "val": 80406000,
//                              "accn": "0001047469-10-001018",
//                              "fy": 2009,
//                              "fp": "FY",
//                              "form": "10-K",
//                              "filed": "2010-02-19",
//                              "frame": "CY2007Q4I"
//                          },
//                          ...
//                      ]
//                }
//          }
//      }
// }
