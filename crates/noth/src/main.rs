use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands::*, TraceLevel};
// use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use dotenv::{dotenv, var};
use noth_warehouse::{self as warehouse, api::Http};
use tokio_postgres::{self as pg, NoTls};
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, info, subscriber, trace, Level};
use tracing_subscriber::FmtSubscriber;

mod cli;

fn preprocess(trace_level: Level) {
    dotenv().ok();
    let my_subscriber = FmtSubscriber::builder()
        .with_max_level(trace_level)
        .finish();
    subscriber::set_global_default(my_subscriber).expect("Set subscriber");
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.trace {
        TraceLevel::DEBUG => Level::DEBUG,
        TraceLevel::INFO => Level::INFO,
        TraceLevel::WARN => Level::WARN,
        TraceLevel::ERROR => Level::ERROR,
    };

    preprocess(log_level);
    trace!("Command line input recorded: {cli:#?}");

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    // cli framework:
    // "> renai <COMMAND>"
    match &cli.command {
        // "> renai fetch [all bulk unzip collection]"
        // run specified steps of data collection process
        Fetch { actions } => {
            use cli::FetchArgs::*;

            for action in actions {
                match action {
                    // ---------------------------------------------------------------------------
                    // "> renai fetch bulks"
                    // download the bulk data zip files
                    Bulks => {
                        use warehouse::schema::stock::index::Sec;
                        info!("Scraping SEC bulk files");
                        let http_client = reqwest::ClientBuilder::new()
                            .user_agent(&var("USER_AGENT")?)
                            .build()?;
                        match Sec::bulk_files(&http_client).await {
                            Ok(_) => trace!("Stock index inserted successfully"),
                            Err(e) => error!("Stock index insert failed: {e}"),
                        }
                    }
                }
            }
        }

        // "> renai rm [buffer]"
        // remove directories
        Rm { directories } => {
            use cli::RmArgs::*;

            if directories.contains(&Buffer) {
                trace!("Removing directory: ./buffer");
                tokio::fs::remove_dir_all("./buffer").await?;
            }

            debug!("Removed directories: {directories:#?}");
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////

        // "> renai insert [stock-index stock-prices stock-metrics crypto-index crypto-prices]"
        // insert datasets to PostgreSQL
        Insert { datasets } => {
            use cli::Dataset::*;

            // open pg connection
            debug!("Establishing PostgreSQL connection");
            let (mut pg_client, pg_conn) = pg::connect(&var("POSTGRES_URL")?, NoTls).await?;
            tokio::spawn(async move {
                if let Err(e) = pg_conn.await {
                    error!("connection error: {}", e);
                }
            });
            debug!("PostgreSQL connection established");

            // parse the Insert action arguments
            for dataset in datasets {
                match dataset {
                    &StockIndex => {
                        use warehouse::schema::stock::index::Sec;

                        // check if ./buffer/submissions.zip exists
                        // if not offer to download it "y/n"

                        info!("Scraping stock index data");
                        match Sec::scrape_index(&mut pg_client).await {
                            Ok(_) => trace!("Stock index inserted successfully"),
                            Err(e) => error!("Stock index insert failed: {e}"),
                        }
                    }

                    // ---------------------------------------------------------------------------
                    &StockPrices => {
                        use warehouse::schema::stock::index::Sec;

                        info!("Scraping stock price data");
                        let http_client = reqwest::ClientBuilder::new()
                            .user_agent(&var("USER_AGENT")?)
                            .build()?;

                        let tickers = Sec::fetch(
                            &http_client,
                            &"https://www.sec.gov/files/company_tickers.json".to_string(),
                        )
                        .await?;

                        let mut stream = stream::iter(tickers.0);
                        while let Some(ticker) = stream.next().await {
                            match Sec::scrape_prices(ticker.clone(), &http_client, &mut pg_client)
                                .await
                            {
                                Ok(_) => {
                                    trace!("Stock prices inserted successfully for {ticker:?}")
                                }
                                Err(e) => {
                                    error!("Stock prices insert failed for {ticker:?}: {e}");
                                    continue;
                                }
                            }
                        }
                    }

                    // ---------------------------------------------------------------------------
                    &StockMetrics => {
                        use warehouse::schema::stock::index::Sec;

                        // check if ./buffer/companyfacts.zip exists
                        // if not offer to download it "y/n"

                        info!("Scraping stock metric data");
                        let http_client = reqwest::ClientBuilder::new()
                            .user_agent(&var("USER_AGENT")?)
                            .build()?;

                        let tickers = Sec::fetch(
                            &http_client,
                            &"https://www.sec.gov/files/company_tickers.json".to_string(),
                        )
                        .await?;

                        let mut stream = stream::iter(tickers.0);
                        while let Some(ticker) = stream.next().await {
                            match Sec::scrape_metrics(ticker.clone(), &mut pg_client).await {
                                Ok(_) => {
                                    trace!("Stock metrics inserted successfully for {ticker:?}")
                                }
                                Err(e) => {
                                    error!("Stock metrics insert failed for {ticker:?}: {e}");
                                    continue;
                                }
                            }
                        }
                    }

                    // ---------------------------------------------------------------------------
                    &CryptoIndex => {
                        use warehouse::schema::crypto::*;

                        info!("Inserting crypto index data");
                        match index::insert(&mut pg_client).await {
                            Ok(_) => trace!("Crypto index inserted succesfully"),
                            Err(e) => error!("Crypto index insert failed: {e}"),
                        }
                    }

                    // ---------------------------------------------------------------------------
                    &CryptoPrices => {
                        use warehouse::schema::crypto::*;

                        info!("Scraping Binance prices");
                        match binance::Binance::scrape(&mut pg_client).await {
                            Ok(_) => {
                                trace!("Binance API finished scraping SPOT prices succesfully")
                            }
                            Err(e) => error!("Binance API scraping failed SPOT prices: {e}"),
                        }

                        info!("Scraping KuCoin prices");
                        match kucoin::KuCoin::scrape(&mut pg_client).await {
                            Ok(_) => {
                                trace!("KuCoin API finished scraping SPOT prices successfully")
                            }
                            Err(e) => error!("KuCoin API scraping failed SPOT prices: {e}"),
                        }
                    }

                    // ---------------------------------------------------------------------------
                    &Econ => {
                        use warehouse::schema::econ;

                        info!("Scraping Fred API");
                        match econ::us::Fred::scrape(&mut pg_client).await {
                            Ok(_) => trace!("US Fred API scraped successfully"),
                            Err(e) => error!("Fred API scraping failed: {e}"),
                        }
                    }
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////////////////

        // "> renai test"
        // used to test functions
        Test => {
            // use renai_pg::schema::stock::index::Tickers;
            //
            // let http_client = reqwest::ClientBuilder::new()
            //     .user_agent(&var("USER_AGENT")?)
            //     .build()?;
            //
            // let stocks: Vec<String> = http_client
            //     .get("https://www.sec.gov/files/company_tickers.json")
            //     .send()
            //     .await?
            //     .json::<Tickers>()
            //     .await?
            //     .0
            //     .into_iter()
            //     .map(|entry| format!("{:>8} | {}", entry.ticker, entry.title.to_uppercase()))
            //     .collect();
            //
            // let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
            //     .with_prompt("Stock Tickers:")
            //     .default(0)
            //     .items(&stocks)
            //     .interact()
            //     .unwrap();
            //
            // println!("Enjoy your {:?}!", stocks[selection]);
        }
    }

    Ok(())
}
