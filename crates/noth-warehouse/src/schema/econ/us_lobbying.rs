use crate::api::*;
use crate::schema::common::convert_date_type;
use chrono::{Datelike, Utc};
use dotenv::var;
use futures::stream::{self, StreamExt};
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, trace};

/////////////////////////////////////////////////////////////////////////////////
//
// Senate Lobbying Disclosure Act (LDA)
// https://lda.senate.gov/api/redoc/v1/#section/About-the-REST-API/Browsable-API
//
/////////////////////////////////////////////////////////////////////////////////

static US_LOBBYING_QUERY: &str = "
    INSERT INTO econ.us_lobbying (
        dated,
        filer_type,
        filing_state,
        filing_country,
        registrant_name,
        registrant_desc,
        registrant_state,
        registrant_country,
        registrant_contact,
        lobbyist_first_name,
        lobbyist_middle_name,
        lobbyist_last_name,
        pacs,
        contr_type,
        contr_type_disp,
        contr_name,
        payee_name,
        honoree_name,
        amount
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
    ON CONFLICT (dated, filer_type, contr_name, payee_name, honoree_name, amount) DO NOTHING
";

pub struct Lda;

impl Lda {
    fn build_client() -> HttpClient {
        reqwest::ClientBuilder::new()
            .user_agent(&var("USER_AGENT").expect("missing user agent"))
            .build()
            .expect("failed to build http client")
    }

    pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
        let http_client = Self::build_client();
        let auth = var("LDA_API")?;

        // loop through the years; starting at 2020
        let curr_yr = Utc::now().year();
        let mut next: Option<String>;
        for yr in 2020..curr_yr {
            next = Some(format!(
                "https://lda.senate.gov/api/v1/contributions/?filing_year={yr}"
            ));

            // if `next` url exists, keep looping
            while let Some(url) = next {
                let time = std::time::Instant::now();

                let response: Contributions = http_client
                    .get(&url)
                    .header("Authorization", &auth)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("failed to fetch: {}", &url);
                        e
                    })?
                    .json()
                    .await
                    .map_err(|e| {
                        error!("failed to parse json: {}", &url);
                        e
                    })?;

                // insert the datatable
                // -------------------------
                let query = pg_client.prepare(US_LOBBYING_QUERY).await?;
                let transaction = std::sync::Arc::new(pg_client.transaction().await?);

                let mut stream = stream::iter(response.results);
                while let Some(x) = stream.next().await {
                    let registrant = x.registrant.unwrap();
                    let lobbyist = x.lobbyist.unwrap();
                    let mut inner_stream = stream::iter(x.contribution_items);
                    while let Some(y) = inner_stream.next().await {
                        let transaction = transaction.clone();
                        match transaction
                            .execute(
                                &query,
                                &[
                                    &convert_date_type(&y.date.unwrap())?,
                                    &x.filer_type_display,
                                    &x.state_display,
                                    &x.country_display,
                                    &registrant.name,
                                    &registrant.description,
                                    &registrant.state_display,
                                    &registrant.country_display,
                                    &registrant.contact_name,
                                    &lobbyist.first_name,
                                    &lobbyist.middle_name,
                                    &lobbyist.last_name,
                                    &x.pacs,
                                    &y.contribution_type,
                                    &y.contribution_type_display,
                                    &y.contributor_name,
                                    &y.payee_name,
                                    &y.honoree_name,
                                    &y.amount,
                                ],
                            )
                            .await
                        {
                            Ok(_) => trace!("inserted"),
                            Err(e) => error!("failed to insert: {}", e),
                        }
                    }
                }

                // unpack the transcation and commit it to the database
                std::sync::Arc::into_inner(transaction)
                    .expect("failed to unpack Transaction from Arc")
                    .commit()
                    .await
                    .map_err(|e| {
                        error!("failed to commit transaction for {}", &url);
                        e
                    })?;

                trace!(
                    "{} inserted. Elapsed time: {} ms",
                    &url,
                    time.elapsed().as_millis()
                );

                // set the next url to scrape
                //
                // if `next == None`, the loop will proceed to the next year
                next = response.next;
            }
        }

        // TEST
        let data: Value = http_client
            .get("https://lda.senate.gov/api/v1/contributions/?page=2")
            .header("Authorization", &var("LDA_API")?)
            .send()
            .await?
            .json()
            .await?;
        println!("{:?}", data);

        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////
//
// Deserialization
//
/////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
struct Contributions {
    next: Option<String>,
    results: Vec<Contribution>,
}

#[derive(Debug, Deserialize)]
struct Contribution {
    filer_type_display: String,
    state_display: String,
    country_display: String,
    registrant: Option<Registrant>,
    lobbyist: Option<Lobbyist>,
    pacs: Vec<String>, // list of associated PACs
    contribution_items: Vec<ContributionItem>,
}

#[derive(Debug, Deserialize)]
struct ContributionItem {
    contribution_type: Option<String>,
    contribution_type_display: Option<String>,
    contributor_name: Option<String>,
    payee_name: Option<String>,
    honoree_name: Option<String>,
    amount: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Registrant {
    name: Option<String>,
    description: Option<String>,
    state_display: Option<String>,
    country_display: Option<String>,
    contact_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Lobbyist {
    first_name: String,
    middle_name: Option<String>,
    last_name: String,
}
