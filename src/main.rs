// =========================================
//  ██▀███  ▓█████  ███▄    █  ▄▄▄       ██▓
// ▓██ ▒ ██▒▓█   ▀  ██ ▀█   █ ▒████▄    ▓██▒
// ▓██ ░▄█ ▒▒███   ▓██  ▀█ ██▒▒██  ▀█▄  ▒██▒
// ▒██▀▀█▄  ▒▓█  ▄ ▓██▒  ▐▌██▒░██▄▄▄▄██ ░██░
// ░██▓ ▒██▒░▒████▒▒██░   ▓██░ ▓█   ▓██▒░██░
// ░ ▒▓ ░▒▓░░░ ▒░ ░░ ▒░   ▒ ▒  ▒▒   ▓▒█░░▓
//   ░▒ ░ ▒░ ░ ░  ░░ ░░   ░ ▒░  ▒   ▒▒ ░ ▒ ░
//   ░░   ░    ░      ░   ░ ░   ░   ▒    ▒ ░
//    ░        ░  ░         ░       ░  ░ ░
// =========================================

use anyhow::Result;
use clap::Parser;
use futures::{stream, StreamExt};
use log::error;
use serde::{Deserialize, Serialize};
use std::env;

use renai::{
    cli::Cli,
    client_ext::{ClientExt, Document},
    endp::{sec, us_company_index as us, yahoo_finance as yf},
    ui,
};

fn preprocess() {
    dotenv::dotenv().ok();
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    preprocess();

    let cli = Cli::parse();
    println!("{:#?}", cli);

    // let client = reqwest::ClientBuilder::new()
    //     .user_agent(&env::var("USER_AGENT")?)
    //     .build()?;

    // fetch bulk core & unzip
    // fetch()?;                    <--- needs writing with rayon
    // sec::unzip().await?;

    // fetch US stock
    // let tickers = Document {
    //     _id: "us_index".to_string(),
    //     _rev: "".to_string(),
    //     data: us::extran(&client).await?,
    // };
    // // client
    // //     .insert_doc(
    // //         &tickers,
    // //         &env::var("DATABASE_URL")?,
    // //         &format!("stock/{}", tickers._id),
    // //     )
    // //     .await
    // //     .expect("failed to insert index doc");

    // // collect data
    // use std::sync::Arc;
    // use tokio::sync::Mutex;
    // let pb = Arc::new(Mutex::new(ui::single_pb(tickers.data.len() as u64)));
    // let stream = stream::iter(tickers.data)
    //     .map(|company| {
    //         let client = &client;
    //         let pb = pb.clone();
    //         async move {
    //             // fetch fundamentals
    //             let core = match sec::extran(&company.cik_str).await {
    //                 Ok(data) => data,
    //                 Err(e) => {
    //                     error!("Failed to fetch fundamentals: {:#?}", e);
    //                     return Err(e);
    //                 }
    //             };

    //             // fetch price
    //             let price = match client
    //                 .fetch_price_data(&company.ticker, &company.title)
    //                 .await
    //             {
    //                 Ok(data) => data,
    //                 Err(e) => {
    //                     error!("Failed to fetch price data: {:#?}", e);
    //                     return Err(e);
    //                 }
    //             };

    //             // build doc
    //             let document = Document {
    //                 _id: company.ticker.clone(),
    //                 _rev: "".to_string(),
    //                 data: StockData { core, price },
    //             };

    //             // upload doc
    //             client
    //                 .insert_doc(
    //                     &document,
    //                     &env::var("DATABASE_URL")
    //                         .expect("failed to retrieve environment variable: DATABASE_URL"),
    //                     &format!("stock/{}", company.ticker),
    //                 )
    //                 .await
    //                 .expect("failed to insert doc");
    //             pb.lock().await.inc(1);
    //             Ok(())
    //         }
    //     })
    //     .buffer_unordered(num_cpus::get());

    // stream
    //     .for_each(|fut| async {
    //         match fut {
    //             Ok(_) => {}
    //             Err(e) => {
    //                 error!("Error processing company: {:#?}", e);
    //             }
    //         }
    //     })
    //     .await;

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////
// Output schema
//////////////////////////////////////////////////////////////////////////////////////

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

pub type CoreSet = Vec<sec::CoreCell>;
// pub type CoreSet = Vec<sec::CoreCell>
// "core": [
//      {
//          "dated": "2021-01-01",
//          "Revenue": 1298973.0,
//          "DilutedEPS": 2.7,
//      },
//      {
//          "dated": "2022-01-01",
//          "Revenue": 23112515.0,
//          "DilutedEPS": 1.72,
//      },
//      ...
// ]

pub type PriceSet = Vec<yf::PriceCell>;
// "price": [
//      {
//          "dated": "2021-01-01",
//          "open": 123.0,
//          "adj_close": 124.2,
//      },
//      {
//          "dated": "2022-01-01",
//          "open": 124.2,
//          "adj_close": 122.0,
//      },
//      ...
// ]
