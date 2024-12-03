use crate::api::*;
use crate::schema::common::convert_date_type;
use async_trait::async_trait;
use dotenv::var;
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::{self as stream, StreamExt};
use tracing::{debug, error, trace};

//////////////////////////////////////////////////////////////////////////////////////

static INSERT_QUERY: &str = "
    INSERT INTO econ.us_std (dated, metric, val) VALUES ($1, $2, $3)
    ON CONFLICT (dated, metric, val) DO NOTHING
";

pub struct Fred;

impl Fred {
    pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let http_client: HttpClient = reqwest::ClientBuilder::new().build()?;
        Self::etl(&http_client, pg_client).await?;
        Ok(())
    }
}

#[async_trait]
impl Api<Observations> for Fred {
    async fn etl(http_client: &HttpClient, pg_client: &mut PgClient) -> anyhow::Result<()> {
        for dataset in ["unemployment rate", "interest rate"] {
            let key = &var("FRED_API")?;
            let url = match dataset {
                "unemployment rate" => 
                    format!("https://api.stlouisfed.org/fred/series/observations?series_id=DFF&api_key={key}&file_type=json"),
                "interest rate" => 
                    format!("https://api.stlouisfed.org/fred/series/observations?series_id=UNRATE&api_key={key}&file_type=json"),
                _ => unreachable!("unreachable dataset in Fred etl")
            };

            trace!("fetching Fred data {dataset}");
            let data = Self::fetch(&http_client, &url).await?;
            trace!("inserting Fred data {dataset}");
            Self::insert(data, pg_client, dataset.to_string()).await?;
        }

        Ok(())
    }
}

//////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Http<Observations> for Fred {
    async fn fetch(http_client: &HttpClient, url: &String) -> anyhow::Result<Observations> {
        Ok(http_client
            .get(url)
            .header("User-Agent", &var("USER_AGENT")?)
            .send()
            .await?
            .json()
            .await?)
    }
}

//////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
impl Postgres<Observations> for Fred {
    type Info = String;

    async fn insert(
        data: Observations,
        pg_client: &mut PgClient,
        additional_info: Self::Info,
    ) -> anyhow::Result<()> {
        let query = pg_client.prepare(INSERT_QUERY).await?;
        let transaction = Arc::new(pg_client.transaction().await?);

        let metric = additional_info;

        let mut stream = stream::iter(data.inner);
        while let Some(obs) = stream.next().await {
            let query = &query;
            let metric = &metric;
            let transaction = transaction.clone();
            async move {
                let dated = convert_date_type(&obs.dated).expect("error converting date type");
                let val = obs.value.parse::<f64>().expect("error parsing value");
                let result = transaction.execute(query, &[&dated, &metric, &val]).await;

                match result {
                    Ok(_) => trace!("inserted US interest rate [{dated}, {metric}, {val}]"),
                    Err(e) => error!("error inserting interest rate: {:?}", e),
                }
            }
            .await;
        }

        match Arc::into_inner(transaction)
            .expect("failed to unpack Transaction from Arc")
            .commit()
            .await
        {
            Ok(_) => debug!("committed transaction for econ.us"),
            Err(e) => error!("error committing transaction for econ.us: {:?}", e),
        }

        Ok(())
    }
}

//////////////////////////////////////////////////////////////////////////////////////

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
