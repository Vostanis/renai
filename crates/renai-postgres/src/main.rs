pub mod schema;

use dotenv::{dotenv, var};
use schema::stock::index::Sec;
use schema::stock::metrics::download_zip_file;
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
        .with_max_level(tracing::Level::DEBUG)
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

    let sec = Sec::new();
    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;
    let http_client = Arc::new(http_client);

    if DOWNLOAD_ZIP {
        download_zip_file().await?;
    }

    let tickers = sec
        .company_tickers
        .get()
        .await
        .expect("Failed to fetch Tickers");

    // # async
    let mut stream = stream::iter(tickers.0);
    while let Some(ticker) = stream.next().await {
        let time = std::time::Instant::now();
        let tckr = ticker.ticker.clone();
        let title = ticker.title.clone();

        let pgclient = pgclient.clone();
        let http_client = http_client.clone();

        async move {
            let mut pgclient = pgclient.lock().await;
            pgclient
                .query(
                    "INSERT INTO stock.index VALUES ($1, $2, $3)",
                    &[&ticker.stock_id, &ticker.ticker, &ticker.title],
                )
                .await
                .expect("failed to insert index");

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

    // # single-threaded
    // for ticker in tickers.0 {
    //     trace!("Processing prices for [{}] {}", ticker.ticker, ticker.title);
    //     ticker
    //         .prices(&http_client, &mut pgclient)
    //         .await
    //         .expect("Failed to process prices");
    //     trace!("Prices processed for [{}] {}", ticker.ticker, ticker.title);
    //
    //     trace!(
    //         "Processing metrics for [{}] {}",
    //         ticker.ticker,
    //         ticker.title
    //     );
    //     ticker
    //         .metrics(&mut pgclient)
    //         .await
    //         .expect("Failed to process metrics");
    //     trace!("Metrics processed for [{}] {}", ticker.ticker, ticker.title);
    // }

    use schema::crypto::binance::Binance;
    // use schema::crypto::kraken::Kraken;
    use schema::crypto::kucoin::KuCoin;

    let mut pgclient = pgclient.lock().await;
    Binance::fetch(&mut pgclient).await?;
    KuCoin::fetch(&mut pgclient).await?;
    // Kraken::fetch(&mut pgclient).await.unwrap();

    Ok(())
}
