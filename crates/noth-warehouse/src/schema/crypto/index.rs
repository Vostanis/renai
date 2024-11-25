use lazy_static::lazy_static;
use std::collections::BTreeMap as Map;
use tracing::{debug, error, trace};

lazy_static! {
    /// Statically defined crypto pairs.
    ///
    /// This is currently maintained manually.
    pub static ref PAIRS: Map<i32, &'static str> = Map::from([
        (0, "BTCUSDT"),
        (1, "ETHUSDT"),
        (2, "SOLUSDT"),
        (3, "SUIUSDT"),
        (4, "ALPHUSDT"),
        (5, "KASUSDT"),
        (6, "BTTUSDT"),
        (7, "TRXUSDT"),
        (8, "ADAUSDT"),
        (9, "XRPUSDT"),
        (10, "BTTUSDT"),
        (11, "BNBUSDT"),
        (12, "AVAXUSDT"),
        (13, "DOTUSDT"),
        (14, "DOGEUSDT"),
        (15, "SHIBUSDT"),
        (16, "TAOUSDT"),
        (17, "PEPEUSDT"),
        (18, "MEWUSDT"),
        (19, "EIGENUSDT"),
    ]);

    /// Query to insert Crypto Index rows.
    pub static ref INDEX_QUERY: &'static str = "
        INSERT INTO crypto.index (crypto_id, pair)
        VALUES ($1, $2)
        ON CONFLICT (crypto_id) DO NOTHING
    ";

    /// Query in to Index Price rows.
    pub static ref PRICE_QUERY: &'static str = "
        INSERT INTO crypto.prices (
            crypto_id, 
            time, 
            interval, 
            opening, 
            high, 
            low, 
            closing, 
            volume, 
            trades, 
            amount,
            source
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (crypto_id, time, interval, source) DO NOTHING
    ";
}

/// Insert crypto pairs into the `crypto.index` table.
pub async fn insert(pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
    let query = pg_client.prepare(&INDEX_QUERY).await?;
    let transaction = pg_client.transaction().await?;

    for (crypto_id, pair) in PAIRS.iter() {
        trace!("inserting crypto pair: {:?}", pair);
        match transaction.execute(&query, &[crypto_id, pair]).await {
            Ok(_) => trace!("inserted crypto pair {:?}", pair),
            Err(e) => error!("error inserting crypto pair {:?} | {:?}", pair, e),
        }
    }

    match transaction.commit().await {
        Ok(_) => debug!("committed transaction for crypto.index"),
        Err(e) => error!("error committing transaction for crypto.index: {:?}", e),
    }

    Ok(())
}
