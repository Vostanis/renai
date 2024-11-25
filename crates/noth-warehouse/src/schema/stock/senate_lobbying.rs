// use crate::api::*;
// use async_trait::async_trait;
// use dotenv::var;
// use serde::Deserialize;
//
// /////////////////////////////////////////////////////////////////////////////////
// //
// // Senate Lobbying Disclosure Act (LDA)
// // https://lda.senate.gov/api/redoc/v1/#section/About-the-REST-API/Browsable-API
// //
// /////////////////////////////////////////////////////////////////////////////////
//
// pub struct Lda;
//
// impl Lda {
//     fn build_client() -> HttpClient {
//         HttpClient::new()
//     }
//
//     pub async fn scrape(pg_client: &mut PgClient) -> anyhow::Result<()> {
//         let http_client = Self::build_client();
//         // <Self as Http<Issues>>::typefetch(&http_client, "").await?;
//         Ok(())
//     }
// }
//
// // -----------------------------------------------------------------------------
// // Government Agencies List
//
// // [
// //     {
// //         "id": 211,
// //         "name": "Access Board"
// //     },
// //     {
// //         "id": 131,
// //         "name": "Administration for Children & Families (ACF)"
// //     },
// //     {
// //         "id": 130,
// //         "name": "Administration on Aging"
// //     },
// type Agencies = Vec<Agency>;
//
// #[derive(Debug, Deserialize)]
// struct Agency {
//     id: u32,
//     name: String,
// }
//
// static AGENCY_QUERY: &str = "
//     INSERT INTO lda.agencies (id, name)
//     VALUES ($1, $2)
//     ON CONFLICT (id) DO NOTHING
// ";
//
// async fn scrape_agencies(http_client: &HttpClient) -> anyhow::Result<Agencies> {
//     let url = "https://lda.senate.gov/api/v1/governmententities/";
//     let data = http_client
//         .get(url)
//         .header("Authorization", format!("Token {}", var("LDA_API")?))
//         .send()
//         .await?
//         .json()
//         .await?;
//     Ok(data)
// }
//
// #[async_trait]
// impl Postgres<Agencies> for Lda {
//     type Info = ();
//
//     async fn insert(
//         data: Issues,
//         pg_client: &mut PgClient,
//         _info: Self::Info,
//     ) -> anyhow::Result<()> {
//         // let
//         Ok(())
//     }
// }
//
// // -----------------------------------------------------------------------------
// // Lobbying Activity Issues List
//
// // [
// //     {
// //         "value": "ACC",
// //         "name": "Accounting"
// //     },
// //     {
// //         "value": "ADV",
// //         "name": "Advertising"
// //     },
// //     {
// //         "value": "AER",
// //         "name": "Aerospace"
// //     },
// type Issues = Vec<Issue>;
//
// #[derive(Debug, Deserialize)]
// struct Issue {
//     value: String,
//     name: String,
// }
//
// async fn scrape_issues(http_client: &HttpClient) -> anyhow::Result<Issues> {
//     let url = "https://lda.senate.gov/api/v1/issues/";
//     let data = http_client
//         .get(url)
//         .header("Authorization", format!("Token {}", var("LDA_API")?))
//         .send()
//         .await?
//         .json()
//         .await?;
//     Ok(data)
// }
//
// #[async_trait]
// impl Postgres<Issue> for Lda {
//     type Info = ();
//
//     async fn insert(
//         data: Issues,
//         pg_client: &mut PgClient,
//         _info: Self::Info,
//     ) -> anyhow::Result<()> {
//         Ok(())
//     }
// }
//
// // ----------------------------------------------------------------------------
//
// // {
// //     "count": 124826,
// //     "next": "https://lda.senate.gov/api/v1/clients/?page=2",
// //     "previous": null,
// //     "results": [
// //         {
// //             "id": 44400,
// //             "url": "https://lda.senate.gov/api/v1/clients/44400/",
// //             "client_id": 44400,
// //             "name": "BLUECROSS BLUESHIELD OF TENNESSEE, INC.",
// //             "general_description": "Health benefit plan company",
// //             "client_government_entity": null,
// //             "client_self_select": false,
// //             "state": "TN",
// //             "state_display": "Tennessee",
// //             "country": "US",
// //             "country_display": "United States of America",
// //             "ppb_state": "TN",
// //             "ppb_state_display": "Tennessee",
// //             "ppb_country": "US",
// //             "ppb_country_display": "United States of America",
// //             "effective_date": "2022-02-09",
// //             "registrant": {
// //                 "id": 401105130,
// //                 "url": "https://lda.senate.gov/api/v1/registrants/401105130/",
// //                 "house_registrant_id": 401105130,
// //                 "name": "BRIDGE PUBLIC AFFAIRS, LLC",
// //                 "description": "Government Relations and Public Affairs",
// //                 "address_1": "P.O. Box 171",
// //                 "address_2": null,
// //                 "address_3": null,
// //                 "address_4": null,
// //                 "city": "Chattanooga",
// //                 "state": "TN",
// //                 "state_display": "Tennessee",
// //                 "zip": "37401",
// //                 "country": "US",
// //                 "country_display": "United States of America",
// //                 "ppb_country": "US",
// //                 "ppb_country_display": "United States of America",
// //                 "contact_name": "PRESLEY ABNEY",
// //                 "contact_telephone": "+1 423-771-4272",
// //                 "dt_updated": "2024-07-26T14:51:59.870940-04:00"
// //             }
// //         },
// #[derive(Debug, Deserialize)]
// struct Clients {
//     count: u32,
//     next: Option<String>,
//     previous: Option<String>,
//     results: Vec<Client>,
// }
//
// #[derive(Debug, Deserialize)]
// struct Client {
//     id: u32,
//     name: String,
//     general_description: String,
//     client_government_entity: Option<String>,
//     #[serde(rename = "state_display")]
//     state: String,
//     country: String,
//     effective_date: String,
//     registrant: Registrant,
// }
//
// #[derive(Debug, Deserialize)]
// struct Registrant {
//     id: u32,
//     name: String,
//     description: String,
//     address_1: Option<String>,
//     address_2: Option<String>,
//     address_3: Option<String>,
//     address_4: Option<String>,
//     city: String,
//     #[serde(rename = "state_display")]
//     state: String,
//     zip: String,
//     country: String,
//     contact_name: String,
// }
//
// impl Http<Clients> for Lda {}
// impl Postgres<Clients> for Lda {}
//
// // ----------------------------------------------------------------------------
//
// // {
// //     "count": 1785074,
// //     "next": "https://lda.senate.gov/api/v1/filings/?page=2",
// //     "previous": null,
// //     "results": [
// //         {
// //             "url": "https://lda.senate.gov/api/v1/filings/455edc06-55d1-41ed-878e-70a4040f953c/",
// //             "filing_uuid": "455edc06-55d1-41ed-878e-70a4040f953c",
// //             "filing_type": "MM",
// //             "filing_type_display": "Mid-Year Report",
// //             "filing_year": 1999,
// //             "filing_period": "mid_year",
// //             "filing_period_display": "Mid-Year (Jan 1 - Jun 30)",
// //             "filing_document_url": "https://lda.senate.gov/filings/public/filing/455edc06-55d1-41ed-878e-70a4040f953c/print/",
// //             "filing_document_content_type": "application/pdf",
// //             "income": null,
// //             "expenses": null,
// //             "expenses_method": null,
// //             "expenses_method_display": null,
// //             "posted_by_name": null,
// //             "dt_posted": "1905-06-24T00:00:00-05:00",
// //             "termination_date": null,
// //             "registrant_country": "United States of America",
// //             "registrant_ppb_country": "United States of America",
// //             "registrant_address_1": null,
// //             "registrant_address_2": null,
// //             "registrant_different_address": null,
// //             "registrant_city": null,
// //             "registrant_state": null,
// //             "registrant_zip": null,
// //             "registrant": {
// //                 "id": 9181,
// //                 "url": "https://lda.senate.gov/api/v1/registrants/9181/",
// //                 "house_registrant_id": 33629,
// //                 "name": "CHURCHILL GROUP",
// //                 "description": null,
// //                 "address_1": "4851 WHITESBURG DR   SUITE G",
// //                 "address_2": null,
// //                 "address_3": null,
// //                 "address_4": null,
// //                 "city": "HUNTSVILLE",
// //                 "state": "AL",
// //                 "state_display": "Alabama",
// //                 "zip": "35802",
// //                 "country": "US",
// //                 "country_display": "United States of America",
// //                 "ppb_country": "US",
// //                 "ppb_country_display": "United States of America",
// //                 "contact_name": "",
// //                 "contact_telephone": "",
// //                 "dt_updated": "2022-01-13T14:47:19.098634-05:00"
// //             },
// //             "client": {
// //                 "id": 113256,
// //                 "url": "https://lda.senate.gov/api/v1/clients/113256/",
// //                 "client_id": 36,
// //                 "name": "AMERICAN FAMILY BUSINESS INST",
// //                 "general_description": null,
// //                 "client_government_entity": null,
// //                 "client_self_select": null,
// //                 "state": "AL",
// //                 "state_display": "Alabama",
// //                 "country": "US",
// //                 "country_display": "United States of America",
// //                 "ppb_state": "AL",
// //                 "ppb_state_display": "Alabama",
// //                 "ppb_country": "US",
// //                 "ppb_country_display": "United States of America",
// //                 "effective_date": "1996-02-14"
// //             },
// //             "lobbying_activities": [
// //                 {
// //                     "general_issue_code": "TAX",
// //                     "general_issue_code_display": "Taxation/Internal Revenue Code",
// //                     "description": null,
// //                     "foreign_entity_issues": null,
// //                     "lobbyists": [
// //                         {
// //                             "lobbyist": {
// //                                 "id": 361,
// //                                 "prefix": null,
// //                                 "prefix_display": null,
// //                                 "first_name": "WAYNE",
// //                                 "nickname": null,
// //                                 "middle_name": null,
// //                                 "last_name": "PARKER",
// //                                 "suffix": null,
// //                                 "suffix_display": null
// //                             },
// //                             "covered_position": "N/A",
// //                             "new": null
// //                         }
// //                     ],
// //                     "government_entities": [
// //                         {
// //                             "id": 2,
// //                             "name": "HOUSE OF REPRESENTATIVES"
// //                         },
// //                         {
// //                             "id": 1,
// //                             "name": "SENATE"
// //                         }
// //                     ]
// //                 }
// //             ],
// //             "conviction_disclosures": [],
// //             "foreign_entities": [],
// //             "affiliated_organizations": []
// //         },
// #[derive(Debug, Deserialize)]
// struct Filings {
//     count: u32,
//     next: Option<String>,
//     previous: Option<String>,
//     results: Vec<Filing>,
// }
//
// #[derive(Debug, Deserialize)]
// struct Filing {
//     #[serde(rename = "filing_document_url")]
//     url: String,
//     #[serde(rename = "filing_type_display")]
//     filing_type: String,
//     filing_document_content_type: String,
//     filing_year: u16,
//     income: String,
//     expenses: String,
//     #[serde(rename = "expenses_method_display")]
//     expenses_method: String,
//     #[serde(rename = "dt_posted")]
//     date_posted: String,
//     termination_date: Option<String>,
//     registrant: Registrant,
//     client: Client,
//     // lobbying_activities: Vec<Act
//     // conviction_disclosures: [],
//     // foreign_entities: [],
//     // affiliated_organizations: []
// }
//
// // ----------------------------------------------------------------------------
//
// // ----------------------------------------------------------------------------
