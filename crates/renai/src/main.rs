use anyhow::Result;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use dotenv::{dotenv, var};
use renai_client::prelude::*;
use renai_postgres::schema::stock::index::Tickers;
use tracing::{debug, error, trace};

mod cli;

fn preprocess() {
    dotenv().ok();

    // initialise logger
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    preprocess();
    let cli = cli::Cli::parse();
    trace!("Command line input recorded: {cli:#?}");

    // cli framework:
    // "> renai <COMMAND>"
    use cli::Commands::*;
    match &cli.command {
        // "> renai fetch [all bulk unzip collection]"
        // run specified steps of data collection process
        Fetch { actions } => {
            use cli::FetchArgs::*;

            if actions.contains(&All) {
                let all_actions = vec![cli::FetchArgs::Bulk, cli::FetchArgs::Unzip];
                process_fetch_args(&all_actions).await?;
            } else {
                process_fetch_args(&actions).await?;
            }
        }

        // "> renai rm [buffer]"
        // remove directories
        Rm { directories } => {
            use cli::RmArgs::*;

            if directories.contains(&Buffer) {
                tokio::fs::remove_dir_all("./buffer").await?;
            }

            debug!("Removing directories: {directories:#?}");
        }

        // "> renai insert [stock-index stock-prices stock-metrics crypto-prices]"
        // insert datasets to PostgreSQL
        Insert { datasets } => {
            use cli::Dataset::*;

            while let Some(dataset) = datasets.iter().next() {
                match dataset {
                    StockIndex => {}
                    StockPrices => {}
                    StockMetrics => {}
                    CryptoPrices => {}
                }
            }
        }

        // "> renai test"
        // used to test functions
        Test => {
            let http_client = reqwest::ClientBuilder::new()
                .user_agent(&var("USER_AGENT")?)
                .build()?;

            let stocks: Vec<String> = http_client
                .get("https://www.sec.gov/files/company_tickers.json")
                .send()
                .await?
                .json::<Tickers>()
                .await?
                .0
                .into_iter()
                .map(|entry| format!("{:>8} | {}", entry.ticker, entry.title.to_uppercase()))
                .collect();

            let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Stock Tickers:")
                .default(0)
                .items(&stocks)
                .interact()
                .unwrap();

            println!("Enjoy your {:?}!", stocks[selection]);
        }
    }

    Ok(())
}

async fn process_dataset_args(datasets: &[cli::Dataset]) -> Result<()> {
    use cli::Dataset::*;
    use tokio_postgres::{self as pg, NoTls};

    // build http_client & pg_conn
    // >> http client
    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;

    // >> pg connection
    let (pg_client, pg_conn) = pg::connect(&var("POSTGRES_URL")?, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = pg_conn.await {
            error!("connection error: {}", e);
        }
    });

    // pre-executables
    // >> stock tickers
    let stocks: Tickers = http_client
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await?
        .json()
        .await?;

    // >> crypto pairs
    let cryptos = vec![
        "BTCUSDT", "ETHUSDT", "SOLUSDT", "SUIUSDT", "KASUSDT", "ALPHUSDT",
    ];

    // execute on each argument
    while let Some(dataset) = datasets.iter().next() {
        match dataset {
            // stocks
            &StockIndex => {
                // Stocks::insert(&tickers, &mut pg_client).await?;
            }
            &StockPrices => {
                // let mut stream = stream::iter(tickers);
                // Stocks::insert(&tickers, &mut pg_client).await?;
            }
            &StockMetrics => {
                // Stocks::insert(&tickers, &mut pg_client).await?;
            }
            // cryptos
            &CryptoPrices => {
                // Binance::get(&cryptos, &mut pg_client).await;
                // KuCoin::get(&cryptos, &mut pg_client).await;
                // Kraken::get(&cryptos, &mut pg_client).await;
            }
        }
    }

    Ok(())
}

async fn process_fetch_args(actions: &[cli::FetchArgs]) -> Result<()> {
    use cli::FetchArgs::*;

    // download bulk SEC file to `./buffer`
    if actions.contains(&Bulk) {
        debug!("Downloading SEC bulk file ...");
        let client = build_client(&var("USER_AGENT")?)?;
        client
            .download_file(
                "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
                "./buffer/companyfacts.zip",
            )
            .await?;
        debug!("SEC bulk file downloaded");
    }

    // unzip bulk SEC file
    if actions.contains(&Unzip) {
        debug!("Unzipping SEC bulk file ...");
        renai_common::fs::unzip("./buffer/companyfacts.zip", "./buffer/companyfacts").await?;
        debug!("SEC bulk file unzipped");
    }

    Ok(())
}
