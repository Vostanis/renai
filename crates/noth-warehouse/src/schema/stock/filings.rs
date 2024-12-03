use crate::schema::common::convert_date_type;
use crate::schema::stock::index::FILINGS_QUERY;
use dotenv::var;
use futures::stream::{self as stream, StreamExt};
use noth_util::fs::read_json;
use serde::de::Deserializer;
use serde::Deserialize;
use tokio_postgres::Client as PgClient;
use tracing::{debug, error, trace};
use url::Url;

//////////////////////////////////////////////////////////////////////////////////////////////////
//
// Scrape SEC Filings to postgres(tsvector)
//
//////////////////////////////////////////////////////////////////////////////////////////////////

// async scrape sec filings, per ticker
// --------------------------------------------------------------------------
//
// >> 1. get filings from:      ./buffer/submissions/CIK0001045810.json
//
// >> 2. get html data from:    https://www.sec.gov/Archives/edgar/data/1045810/000104581024000316/nvda-20241027.htm
//                              https://www.sec.gov/Archives/edgar/data/{cik}/{accessionNumber}/{primary_document}
//
pub async fn scrape_sec_filings(pg_client: &mut PgClient) -> anyhow::Result<()> {
    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;

    // loop the files in submissions directory
    let dir = std::fs::read_dir("./buffer/submissions")?;
    for file in dir {
        // --------------------------------------------------------------------------

        // 1a. read json file
        let path = file?.path().to_str().expect("read file path").to_string();
        let filings: Submissions = read_json(&path).await.map_err(|e| {
            error!("failed to read file {}", &path);
            e
        })?;

        // 1b. parse documents
        let base = filings.filings.recent;
        let acc_nums = &base.accession_number;
        let docs = &base.primary_document;
        let form = &base.form;
        let date = &base.report_date;
        let cik = &filings.cik;
        let name = &filings.name;
        let urls: Vec<(String, &String, &String)> = acc_nums
            .iter()
            .zip(docs)
            .zip(form)
            .zip(date)
            .map(|(((acc_num, doc), form), date)| {
                (
                    format!(
                        "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
                        cik, acc_num, doc
                    ),
                    form,
                    date,
                )
            })
            .collect();
        let full_cik = format!("{:010}", cik);

        // --------------------------------------------------------------------------

        debug!("stream over endpoints for: {name} - CIK{full_cik}");
        let stream = stream::iter(urls)
            .map(|(url, formtype, date)| {
                let pg_client = &pg_client;
                let http_client = &http_client;
                let filename: String = Url::parse(&url)
                    .unwrap()
                    .path_segments()
                    .unwrap()
                    .last()
                    .map(|s| s.to_string())
                    .unwrap();
                let cik = &full_cik;
                let url = url.clone();

                async move {
                    let time = std::time::Instant::now();

                    // 2a. GET file
                    trace!("http GET requesting {url}");
                    let filing = http_client
                        .get(&url)
                        .send()
                        .await
                        .map_err(|e| {
                            error!("failed to GET {filename}: {e}");
                            e
                        })
                        .expect("GET filing")
                        .text()
                        .await
                        .expect("turn filing to text");

                    // 2b. pg insert
                    trace!("inserting to postgres {filename}");
                    let result = pg_client
                        .query(
                            FILINGS_QUERY,
                            &[
                                &cik,
                                &{
                                    // handles empty dates
                                    match date.len() {
                                        0 => None,
                                        _ => Some(
                                            convert_date_type(date).expect("convert date type"),
                                        ),
                                    }
                                },
                                &filename,
                                &formtype,
                                &url,
                                &filing,
                            ],
                        )
                        .await;

                    match result {
                        Ok(_) => trace!("{filename} inserted"),
                        Err(e) => error!("failed to insert {filename}, {e}"),
                    }

                    // 2c. finish waiting 1s
                    let elapsed = time.elapsed().as_millis();
                    if elapsed < 1000 {
                        let wait = 1000 - elapsed;
                        trace!("{filename} is waiting {wait}ms");
                        let _ = std::time::Duration::from_millis(wait.try_into().unwrap());
                    }
                }
            })
            .buffered(10);

        let _result: Vec<_> = stream.collect().await;

        // --------------------------------------------------------------------------

        pg_client
            .query(
                "CREATE INDEX idx_content_tsvector ON stock.filings USING GIN (content_ts)",
                &[],
            )
            .await?;
    }

    Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//
// Deserialization
//
//////////////////////////////////////////////////////////////////////////////////////////////////

// {
//      "cik":"1045810",
//      "entityType":"operating",
//      "sic":"3674",
//      "sicDescription":"Semiconductors & Related Devices",
//      "ownerOrg":"04 Manufacturing",
//      "insiderTransactionForOwnerExists":1,
//      "insiderTransactionForIssuerExists":1,
//      "name":"NVIDIA CORP",
//      "tickers":["NVDA"],
//      "exchanges":["Nasdaq"],
//      "ein":"943177549",
//      "description":"",
//      "website":"",
//      "investorWebsite":"",
//      "category":"Large accelerated filer",
//      "fiscalYearEnd":"0126",
//      "stateOfIncorporation":"DE",
//      "stateOfIncorporationDescription":"DE",
//      "addresses": {
//           "mailing": {
//               "street1":"2788 SAN TOMAS EXPRESSWAY",
//               "street2":null,
//               "city":"SANTA CLARA",
//               "stateOrCountry":"CA",
//               "zipCode":"95051",
//               "stateOrCountryDescription":"CA"
//           },
//           "business": {
//               "street1":"2788 SAN TOMAS EXPRESSWAY",
//               "street2":null,
//               "city":"SANTA CLARA",
//               "stateOrCountry":"CA",
//               "zipCode":"95051",
//               "stateOrCountryDescription":"CA"
//           }
//       },
//       "phone":"408-486-2000",
//       "flags":"",
//       "formerNames": [
//           {
//               "name":"NVIDIA CORP/CA",
//               "from":"1998-05-07T00:00:00.000Z",
//               "to":"2002-06-04T00:00:00.000Z"
//           }
//       ],
//       "filings": {
//           "recent": {
//               "accessionNumber": [
//                   "0001045810-24-000316",
//                   "0001045810-24-000315",
//                   "0001045810-24-000305",
//                   "0001045810-24-000302",
//               ],
//               "reportDate": [
//                   "2024-10-27",
//                   "2024-11-20",
//                   "2024-09-30",
//                   "2024-11-07",
//                   "",
//               ],
//               "acceptanceDateTime": [
//                   "2024-11-20T16:31:22.000Z",
//                   ...
//               ],
//               "act": [
//                   "33",
//                   "34",
//                   ""
//               ],
//               "form": [
//                   "4",
//                   "144",
//                   "8-K",
//                   "10-K",
//                   "144/A",
//                   "SC 13G/A",
//                   ""
//               ],
//               "primaryDocument": [
//                   "nvda-20241027.htm",
//                   "nvda-20241120.htm",
//                   "xslForm13F_X02/primary_doc.xml",
//                   "xslF345X02/wk-form3_1731448005.xml",
//                   "filing.txt",
//                   "nvda-20241107.htm",
//                   "filename1.pdf",
//               ],
//
#[derive(Debug, Deserialize)]
struct Submissions {
    filings: Filings,
    cik: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Filings {
    recent: FilingMetaData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilingMetaData {
    #[serde(deserialize_with = "de_accession_number")]
    accession_number: Vec<String>,
    report_date: Vec<String>,
    // act: Vec<String>,
    form: Vec<String>,
    primary_document: Vec<String>,
}

// Remove all the `-`s from the accession number
fn de_accession_number<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: Vec<String> = Deserialize::deserialize(deserializer)?;
    let result: Vec<String> = value.iter().map(|x| x.replace("-", "")).collect();
    Ok(result)
}
