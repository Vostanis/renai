use lazy_static::lazy_static;
use std::collections::BTreeMap as Map;
use tracing::{debug, error, trace};

lazy_static! {
    /// Statically defined crypto pairs.
    ///
    /// This is currently maintaned manually.
    pub static ref PAIRS: Map<i32, &'static str> = {
        let mut map = Map::new();
        map.insert(0, "BTCUSDT");
        map.insert(1, "ETHUSDT");
        map.insert(2, "SOLUSDT");
        map.insert(3, "SUIUSDT");
        map.insert(4, "ALPHUSDT");
        map.insert(5, "KASUSDT");
        map.insert(6, "BTTUSDT");
        map
    };
}

/// Insert crypto pairs into the `crypto.index` table.
pub async fn insert(pg_client: &mut tokio_postgres::Client) -> anyhow::Result<()> {
    let query = pg_client
        .prepare(
            "
            INSERT INTO crypto.index (crypto_id, pair)
            VALUES ($1, $2)",
        )
        .await?;

    let transaction = pg_client.transaction().await?;

    for (crypto_id, pair) in PAIRS.iter() {
        trace!("inserting crypto pair: {:?}", pair);
        match transaction.execute(&query, &[crypto_id, pair]).await {
            Ok(_) => trace!("inserted crypto pair: {:?}", pair),
            Err(e) => error!("error inserting crypto pair: {:?}", e),
        }
    }

    match transaction.commit().await {
        Ok(_) => debug!("committed transaction for crypto.index"),
        Err(e) => error!("error committing transaction for crypto.index: {:?}", e),
    }

    Ok(())
}
