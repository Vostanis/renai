use anyhow::Result;
use clap::Parser;
use renai_client::prelude::*;
use renai_postgres::Migrator;

mod cli;

fn preprocess() {
    // grant access to .env
    dotenv::dotenv().ok();

    // initialise logger
    env_logger::init();
}

#[tokio::main]
async fn main() -> Result<()> {
    preprocess();
    let cli = cli::Cli::parse();
    log::info!("Command line input recorded: {cli:#?}");

    // cli framework:
    // "> renai <COMMAND>"
    match &cli.command {
        // "> renai fetch-all"
        // run all steps of data collection process (SHORTCUT)
        cli::Commands::FetchAll => {
            let all_actions = vec![
                cli::FetchArgs::Bulk,
                cli::FetchArgs::Unzip,
                cli::FetchArgs::Collection,
            ];
            process_fetch_args(&all_actions).await?;
        }

        // "> renai fetch [bulk unzip collection]"
        // run specified steps of data collection process
        cli::Commands::Fetch { actions } => {
            process_fetch_args(actions).await?;
        }

        // "> renai rm [buffer]"
        // remove directories
        cli::Commands::Rm { directories } => {
            if directories.contains(&cli::RmArgs::Buffer) {
                tokio::fs::remove_dir_all("./buffer").await?;
            }
            log::info!("Removing directories: {directories:#?}");
        }

        // "> renai migrate [stocks]"
        // migrate schemas from CouchDB to PostgreSQL
        #[allow(unused_variables)]
        cli::Commands::Migrate { schema } => {}

        // "> renai test"
        // used to test functions
        cli::Commands::Test => {
            // let db = renai_fs::db::Database::new(
            //     "admin:password@localhost:5984"
            // )?;
            // db.fetch([
            //     "stocks",
            // ].to_vec()).await?;

            let migr = Migrator::connect().await?;
            migr.migrate_stocks().await?;
        }
    }

    Ok(())
}

async fn process_fetch_args(actions: &[cli::FetchArgs]) -> Result<()> {
    // download bulk SEC file to `./buffer`
    if actions.contains(&cli::FetchArgs::Bulk) {
        log::info!("Downloading SEC bulk file ...");
        let client = build_client(&std::env::var("USER_AGENT")?)?;
        client
            .download_file(
                "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
                "./buffer/companyfacts.zip",
            )
            .await?;
        log::info!("SEC bulk file downloaded");
    }

    // unzip bulk SEC file
    if actions.contains(&cli::FetchArgs::Unzip) {
        log::info!("Unzipping SEC bulk file ...");
        renai_common::fs::unzip(
            "./buffer/companyfacts.zip",
            "./buffer/companyfacts"
        ).await?;
        log::info!("SEC bulk file unzipped");
    }

    // collect price & core data, and upload it
    // if actions.contains(&cli::FetchArgs::Collection) {
    //     let client = build_client()?;
    //     client.mass_collection().await?;
    // }

    Ok(())
}
