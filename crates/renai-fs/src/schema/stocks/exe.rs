use super::{core, index, price::yahoo_finance};
use anyhow::Result;
use futures::StreamExt;
use renai_client::prelude::*;
use serde::{Deserialize, Serialize};

/// Execute stock collection files:
///     1. Collect the index of Company Tickers, from SEC filings;
///     2. a) Fetch core data from SEC bulk file;
///        b) Fetch price data from Yahoo! Finance URL.
pub async fn exe(client: &Client) -> Result<()> {
    let conn = &std::env::var("DATABASE_URL")?;

    // fetch and insert index
    let index = doc!("index", index::us::fetch(client).await?);
    client.insert_doc(&index, conn, "stock/index").await?;
    println!("US index fetched");

    // async iterate through index of tickers and fetch (price & core), then insert to doc
    let stream = futures::stream::iter(index.data)
        .map(|company| {
            // let pb = pb.clone();
            async move {
                // fetch fundamentals
                let core = match core::us::fetch(&company.cik_str).await {
                    Ok(data) => data,
                    Err(e) => {
                        log::error!("Failed to fetch fundamentals: {:#?}", e);
                        return Err(e);
                    }
                };

                // fetch price
                let price =
                    match yahoo_finance::fetch(&client, &company.ticker, &company.title).await {
                        Ok(data) => data,
                        Err(e) => {
                            log::error!("Failed to fetch price data: {:#?}", e);
                            return Err(e);
                        }
                    };

                // build doc
                let company_data = doc!(company.ticker.clone(), StockDataset { core, price });

                // upload doc
                client
                    .insert_doc(&company_data, conn, &format!("stock/{}", company.ticker))
                    .await
                    .expect("failed to insert doc");
                // pb.lock().await.inc(1);

                println!("[{}] {} inserted", &company.ticker, &company.title);
                Ok(())
            }
        })
        .buffer_unordered(num_cpus::get());

    stream
        .for_each(|fut| async {
            match fut {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error processing company: {:#?}", e);
                }
            }
        })
        .await;

    Ok(())
}

/// Final output data collection for a single stock (e.g., Apple, Nvidia, Meta, etc.).
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

/// Core data collection (e.g., Revenue, EPS, Debt, etc.).
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
pub type CoreSet = Vec<super::core::us::CoreCell>;

/// Price data collection (i.e., Open, High, Low, Close, Adj. Close).
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
pub type PriceSet = Vec<super::price::yahoo_finance::PriceCell>;
