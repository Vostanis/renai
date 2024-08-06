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
use futures::{
    stream::{self, Unzip},
    StreamExt,
};
use std::env;

use renai::{
    cli::{self, FetchArgs},
    client_ext::{ClientExt, Document},
    endp::{sec, us_company_index as us, yahoo_finance as yf},
    ui,
};

fn preprocess() {
    dotenv::dotenv().ok();
    env_logger::init();
}

async fn client() -> Result<reqwest::Client> {
    let client = reqwest::ClientBuilder::new()
        .user_agent(&env::var("USER_AGENT")?)
        .build()?;
    Ok(client)
}

#[tokio::main]
async fn main() -> Result<()> {
    preprocess();

    let cli = cli::Cli::parse();
    log::info!("Command line input recorded: {cli:#?}");

    // cli framework
    match &cli.command {
        // run all steps of data collection process (SHORTCUT)
        cli::Commands::FetchAll => {
            let all_actions = vec![
                cli::FetchArgs::Bulk,
                cli::FetchArgs::Unzip,
                cli::FetchArgs::Collection,
            ];
            process_fetch_args(&all_actions).await?;
        }

        // run specified steps of data collection process
        cli::Commands::Fetch { actions } => {
            process_fetch_args(actions).await?;
        }

        // remove directories
        cli::Commands::Rm { directories } => {
            log::info!("Removing directories: {directories:#?}"); // <--- todo!
        }
    }

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

async fn process_fetch_args(actions: &[cli::FetchArgs]) -> Result<()> {
    // download bulk SEC file to `./buffer`
    if actions.contains(&cli::FetchArgs::Bulk) {
        log::info!("Downloading SEC bulk file ...");
        // fetch()?; <--- needs writing with rayon
        log::info!("SEC bulk file downloaded");
    }

    // unzip bulk SEC file
    if actions.contains(&cli::FetchArgs::Unzip) {
        log::info!("Unzipping SEC bulk file ...");
        sec::unzip().await?;
        log::info!("SEC bulk file unzipped");
    }

    // collect price & core data, and upload it
    if actions.contains(&cli::FetchArgs::Collection) {
        let client = client().await?;
        client.mass_collection().await?;
    }

    Ok(())
}
