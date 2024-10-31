pub mod fs;
pub mod schema;

use dotenv::{dotenv, var};
use fs::download_zip_file;
use schema::stock::index;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{self as pg, NoTls};
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, subscriber};
use tracing_subscriber::FmtSubscriber;

// temp static
static DOWNLOAD_ZIP: bool = false;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let my_subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    subscriber::set_global_default(my_subscriber)?;

    // // open pg connection
    let (pgclient, pgconn) = pg::connect(&var("POSTGRES_URL")?, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = pgconn.await {
            error!("connection error: {}", e);
        }
    });

    let pgclient = Arc::new(Mutex::new(pgclient));

    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;
    let http_client = Arc::new(http_client);

    if DOWNLOAD_ZIP {
        download_zip_file(&http_client).await?;
    }

    let tickers = index::Tickers::fetch(&http_client.clone()).await?;
    let mut pg_client = pgclient.lock().await;
    tickers.insert(&mut pg_client).await?;

    // async
    let mut stream = stream::iter(&tickers.0);
    while let Some(ticker) = stream.next().await {
        let time = std::time::Instant::now();
        let tckr = ticker.ticker.clone();
        let title = ticker.title.clone();

        let pgclient = pgclient.clone();
        let http_client = http_client.clone();

        async move {
            let mut pgclient = pgclient.lock().await;
            ticker
                .prices(&http_client, &mut pgclient)
                .await
                .expect("failed to process prices");

            ticker
                .metrics(&mut pgclient)
                .await
                .expect("failed to process metrics");
        }
        .await;

        debug!(
            "[{}] {} collected - elapsed time: {}",
            tckr,
            title,
            time.elapsed().as_millis()
        );
    }

    // use schema::crypto::binance::Binance;
    // // use schema::crypto::kraken::Kraken;
    // use schema::crypto::kucoin::KuCoin;
    //
    // let mut pgclient = pgclient.lock().await;
    // Binance::fetch(&mut pgclient).await?;
    // KuCoin::fetch(&mut pgclient).await?;
    // Kraken::fetch(&mut pgclient).await.unwrap();

    Ok(())
}
