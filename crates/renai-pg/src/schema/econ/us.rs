use crate::schema::common::convert_date_type;
use dotenv::var;
use serde::Deserialize;
use tracing::{debug, error, trace};

/// FRED data api
async fn fetch_fred(
    http_client: &reqwest::Client,
) -> anyhow::Result<(Vec<Observation>, Vec<Observation>)> {
    // DGS10 = 10 yr market yield rate
    let interest_url = format!("https://api.stlouisfed.org/fred/series/observations?series_id=DFF&api_key={}&file_type=json", var("FRED_API")?);
    let unemployment_url = format!("https://api.stlouisfed.org/fred/series/observations?series_id=UNRATE&api_key={}&file_type=json", var("FRED_API")?);
    Ok((
        http_client // interest rate
            .get(interest_url)
            .send()
            .await?
            .json::<Observations>()
            .await?
            .inner,
        http_client // unemployment rate
            .get(unemployment_url)
            .send()
            .await?
            .json::<Observations>()
            .await?
            .inner,
    ))
}

/// Senate Lobbying
// async fn fetch_lda(http_client: &reqwest::Client) -> anyhow::Result<()> {
//     let url = "";
//     let response: serde_json::Value = http_client
//         .get(url)
//         .header("ApiKeyAuth", var("LDA_API")?)
//         .send()
//         .await?
//         .json()
//         .await?;
//     Ok(response)
// }

pub async fn insert(
    http_client: &reqwest::Client,
    pg_client: &mut tokio_postgres::Client,
) -> anyhow::Result<()> {
    debug!("inserting econ.us data");

    // Fred data
    let (interest, unemployment) = fetch_fred(http_client).await.map_err(|e| {
        error!("error fetching econ.us data: {:?}", e);
        e
    })?;

    let query = pg_client
        .prepare("INSERT INTO econ.us (dated, metric, val) VALUES ($1, $2, $3)")
        .await?;
    let transaction = pg_client.transaction().await?;

    // US interest rate
    for obs in interest {
        let dated = convert_date_type(&obs.dated)?;
        let metric = "interest rate";
        let val = obs.value.parse::<f64>()?;
        match transaction.execute(&query, &[&dated, &metric, &val]).await {
            Ok(_) => trace!("inserted US interest rate"),
            Err(e) => error!("error inserting interest rate: {:?}", e),
        }
    }

    // US unemployment rate
    for obs in unemployment {
        let dated = convert_date_type(&obs.dated)?;
        let metric = "unemployment rate";
        let val = obs.value.parse::<f64>()?;
        match transaction.execute(&query, &[&dated, &metric, &val]).await {
            Ok(_) => trace!("inserted US unemployment rate"),
            Err(e) => error!("error inserting interest rate: {:?}", e),
        }
    }

    match transaction.commit().await {
        Ok(_) => debug!("committed transaction for econ.us"),
        Err(e) => error!("error committing transaction for econ.us: {:?}", e),
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Observations {
    #[serde(rename = "observations")]
    pub inner: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
pub struct Observation {
    #[serde(rename = "date")]
    pub dated: String,
    pub value: String,
}
