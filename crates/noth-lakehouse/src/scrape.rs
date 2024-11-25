use dotenv::var;
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::Elasticsearch;
use futures::stream::{self, StreamExt};
use noth_util::fs::read_json;
use noth_warehouse::schema::stock::index::Tickers;
use serde::Deserialize;
// use tracing::{debug, error, info, trace};
use url::Url;

//////////////////////////////////////////////////////////////////////////////////////////////////
//
// Scrape SEC Filings to elasticsearch database
//
//////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn scrape_sec_filings() -> anyhow::Result<()> {
    // scrape tickers
    let http_client = reqwest::ClientBuilder::new()
        .user_agent(&var("USER_AGENT")?)
        .build()?;

    let tickers: Tickers = http_client
        .get("https://www.sec.gov/files/company_tickers.json")
        .send()
        .await?
        .json()
        .await?;

    // async scrape sec filings, per ticker
    // --------------------------------------------------------------------------
    //
    // >> 1. get filings from:      https://data.sec.gov/submissions/CIK0001045810.json
    //                              https://data.sec.gov/submissions/CIK{ten_digit_cik}.json
    //
    // >> 2. get html data from:    https://www.sec.gov/Archives/edgar/data/1045810/000104581024000316/nvda-20241027.htm
    //                              https://www.sec.gov/Archives/edgar/data/{cik}/{accessionNumber}/{lowercase_ticker}-{reportDate}.htm
    //
    let url = Url::parse("https://localhost:9200")?;
    let conn_pool = SingleNodeConnectionPool::new(url);
    let transport = TransportBuilder::new(conn_pool).disable_proxy().build()?;
    let es_client = Elasticsearch::new(transport);

    let manifest_dir = &var("CARGO_MANIFEST_DIR")?;
    let mut manifest_dir = std::path::PathBuf::from(manifest_dir);

    // for each ticker, execute GET & POST
    let stream = stream::iter(tickers.0)
        .for_each_concurrent(10, |ticker| {
            let time = tokio::time::Instant::now();
            let cik = ticker.stock_id;
            let path = format!("./buffer/submissions/CIK{}.json", cik);
            let http_client = &http_client;
            let es_client = &es_client;
            async move {
                // GET
                let filings: Filings = read_json(&path).await.unwrap();

                // POST
                //
                // >>> elasticsearch insert <<<

                let time_elapsed = time.elapsed().as_secs();
                if 1 - time_elapsed > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1 - time_elapsed)).await;
                }
            }
        })
        .await;

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
}

#[derive(Debug, Deserialize)]
struct Filings {
    recent: FilingMetaData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilingMetaData {
    accession_number: Vec<String>,
    report_date: Vec<String>,
    act: Vec<String>,
    form: Vec<String>,
    primary_document: Vec<String>,
}
