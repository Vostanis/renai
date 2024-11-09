pub mod fs;
pub mod schema;

use dotenv::{dotenv, var};
use fs::download_zip_file;
use schema::crypto;
use schema::econ;
use schema::stock::index;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{self as pg, NoTls};
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, subscriber};
use tracing_subscriber::FmtSubscriber;

// temp static
static DOWNLOAD_ZIP: bool = false;

// https://query1.finance.yahoo.com/v8/finance/chart/AAPL?symbol=AAPL&interval=1m&range=max&events=div|split|capitalGains

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // preprocess
    dotenv().ok();
    let my_subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    subscriber::set_global_default(my_subscriber)?;

    // open pg connection
    let (pg_client, pg_conn) = pg::connect(&var("POSTGRES_URL")?, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = pg_conn.await {
            error!("connection error: {}", e);
        }
    });

    let pg_client = Arc::new(Mutex::new(pg_client));

    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;
    let http_client = Arc::new(http_client);

    // if DOWNLOAD_ZIP {
    //     download_zip_file(&http_client).await?;
    // }
    //
    let tickers = index::Tickers::fetch(&http_client.clone()).await?;
    // let mut pg_client = pgclient.lock().await;
    // tickers.insert(&mut pg_client).await?;

    // async
    let mut stream = stream::iter(&tickers.0);
    while let Some(ticker) = stream.next().await {
        let time = std::time::Instant::now();
        let tckr = ticker.ticker.clone();
        let title = ticker.title.clone();

        let pg_client = pg_client.clone();
        let http_client = http_client.clone();

        async move {
            let mut pg_client = pg_client.lock().await;
            ticker
                .prices(&http_client, &mut pg_client)
                .await
                .expect("failed to process prices");

            // ticker
            //     .metrics(&mut pg_client)
            //     .await
            //     .expect("failed to process metrics");
        }
        .await;

        debug!(
            "[{}] {} collected - elapsed time: {}",
            tckr,
            title,
            time.elapsed().as_millis()
        );
    }

    // let mut pg_client = pg_client.lock().await;
    // crypto::index::insert(&mut pg_client).await?;
    // crypto::Binance::fetch(&mut pgclient).await?;
    // crypto::KuCoin::fetch(&mut pgclient).await?;
    // crypto::Kraken::fetch(&mut pgclient).await.unwrap();

    // econ::us::insert(&http_client, &mut pg_client).await?;

    Ok(())
}
