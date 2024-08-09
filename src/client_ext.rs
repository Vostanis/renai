use crate::endp::sec;
use crate::endp::us_company_index as us;
use crate::endp::yahoo_finance as yf;
use crate::schema;
use crate::ui;
use crate::www;
use anyhow::Result;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::Future;
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
    time::sleep,
};

const CHUNK_SIZE: u64 = 100 * 1024 * 1024; // 100 MB

/// Used in (de)serializing document transfers in the
/// CouchDB protocol; see [`insert_doc()`] for more.
///
/// [`insert_doc()`]: ./trait.ClientExt.html#method.insert_doc
#[derive(Deserialize, Serialize, Debug, Clone)]
struct CouchDocument {
    _id: String,
    _rev: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Document<T> {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _rev: String,
    pub data: T,
}

pub trait ClientExt {
    fn insert_doc<T>(
        &self,
        data: &T,
        conn: &str,
        doc_id: &str,
    ) -> impl Future<Output = Result<()>> + Send
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Sync;

    fn fetch_price(
        &self,
        ticker: &String,
        title: &String,
    ) -> impl Future<Output = Result<Vec<yf::PriceCell>>> + Send;

    fn mass_collection(&self) -> impl Future<Output = Result<()>> + Send;

    fn download_file(&self, url: &str, path: &str) -> impl Future<Output = Result<()>> + Send;

    fn download_chunk(
        &self,
        url: &str,
        start: u64,
        end: u64,
        output_file: &mut File,
    ) -> impl Future<Output = Result<()>> + Send;
}

/// Add-on methods for [`reqwest::Client`].
///
/// [`reqwest::Client`]: https://docs.rs/reqwest/latest/reqwest/struct.Client.html
impl ClientExt for Client {
    async fn insert_doc<T>(&self, data: &T, conn: &str, doc_id: &str) -> Result<()>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Sync,
    {
        // check if the document already exists with a GET request
        let conn = format!("{conn}/{doc_id}");
        let client = self;
        let response = client
            .get(conn.clone())
            .send()
            .await
            .expect("failed to retrieve GET response");

        match response.status() {
            // "if the file already exists ..."
            reqwest::StatusCode::OK => {
                // retrieve current Revision ID
                let text = response
                    .text()
                    .await
                    .expect("failed to translate response to text");
                let current_file: CouchDocument = serde_json::from_str(&text)
                    .expect("failed to read current revision to serde schema");

                // PUT the file up with current Revision ID
                let mut updated_file = json!(data);
                updated_file["_rev"] = json!(current_file._rev);
                let _second_response = client
                    .put(conn)
                    .json(&updated_file)
                    .send()
                    .await
                    .expect("failed to retrieve PUT response");
            }

            // "if the file does not exist ..."
            reqwest::StatusCode::NOT_FOUND => {
                // PUT the new file up, requiring no Revision ID (where we use an empty string)
                let new_file = json!(data);
                let _second_response = client
                    .put(conn)
                    .json(&new_file)
                    .send()
                    .await
                    .expect("failed to retrieve PUT response");
            }

            _ => eprintln!("Unexpected status code found - expected `OK` or `NOT_FOUND`"),
        }
        Ok(())
    }

    /// Fetch the price data of a single stock.
    async fn fetch_price(&self, ticker: &String, title: &String) -> Result<Vec<yf::PriceCell>> {
        let price_url = www::price_url(ticker).await;
        let price = {
            let price_response: yf::PriceHistory = self.get(price_url).send().await?.json().await?;

            match price_response.chart.result {
                Some(data) => {
                    let base = &data[0];
                    let price = &base.indicators.quote[0];
                    let adjclose = &base.indicators.adjclose[0].adjclose;
                    let dates = &base.dates;
                    price
                        .open
                        .iter()
                        .zip(price.high.iter())
                        .zip(price.low.iter())
                        .zip(price.close.iter())
                        .zip(price.volume.iter())
                        .zip(adjclose.iter())
                        .zip(dates.iter())
                        .map(
                            |((((((open, high), low), close), volume), adj_close), date)| {
                                yf::PriceCell {
                                    dated: date.clone(),
                                    open: *open,
                                    high: *high,
                                    low: *low,
                                    close: *close,
                                    adj_close: *adj_close,
                                    volume: *volume,
                                }
                            },
                        )
                        .collect::<Vec<_>>()
                }

                None => {
                    log::warn!("[{ticker}] {title} failed to extract Price data; filling with an empty array instead");
                    vec![] // return an empty vec in the absence of actual dataset
                }
            }
        };
        Ok(price)
    }

    /// Get, collect, & upload all available datasets.
    async fn mass_collection(&self) -> Result<()> {
        // fetch US stock
        let tickers = Document {
            _id: "us_index".to_string(),
            _rev: "".to_string(),
            data: us::extran(&self).await?,
        };
        let _upload_index = &self
            .insert_doc(
                &tickers,
                &std::env::var("DATABASE_URL")?,
                &format!("stock/{}", tickers._id),
            )
            .await
            .expect("failed to insert index doc");

        // collect data
        let pb = Arc::new(Mutex::new(ui::single_pb(tickers.data.len() as u64)));
        let stream = futures::stream::iter(tickers.data)
            .map(|company| {
                let client = &self;
                let pb = pb.clone();
                async move {
                    // fetch fundamentals
                    let core = match sec::extran(&company.cik_str).await {
                        Ok(data) => data,
                        Err(e) => {
                            log::error!("Failed to fetch fundamentals: {:#?}", e);
                            return Err(e);
                        }
                    };

                    // fetch price
                    let price = match client.fetch_price(&company.ticker, &company.title).await {
                        Ok(data) => data,
                        Err(e) => {
                            log::error!("Failed to fetch price data: {:#?}", e);
                            return Err(e);
                        }
                    };

                    // build doc
                    let document = Document {
                        _id: company.ticker.clone(),
                        _rev: "".to_string(),
                        data: schema::StockData { core, price },
                    };

                    // upload doc
                    client
                        .insert_doc(
                            &document,
                            &std::env::var("DATABASE_URL")
                                .expect("failed to retrieve environment variable: DATABASE_URL"),
                            &format!("stock/{}", company.ticker),
                        )
                        .await
                        .expect("failed to insert doc");
                    pb.lock().await.inc(1);
                    Ok(())
                }
            })
            .buffer_unordered(num_cpus::get());

        stream
            .for_each(|fut| async {
                match fut {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error processing company: {:#?}", e);
                    }
                }
            })
            .await;

        Ok(())
    }

    /// GET request a file from `url` and write it to `path`, parallelising
    /// the download process with [`rayon`].
    ///
    /// [`rayon`]: https://docs.rs/rayon/latest/rayon/
    async fn download_file(&self, url: &str, path: &str) -> Result<()> {
        use reqwest::header::CONTENT_LENGTH;

        let client = self;

        // Get the content length from the URL header
        let response = client.get(url).send().await?;
        let file_size = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|len| len.to_str().ok())
            .and_then(|len| len.parse::<u64>().ok())
            .unwrap_or(0);

        // Build a progress bar
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#|-"),
        );
        let pb = Arc::new(pb);

        // Ensure the directory exists
        let dir_path = std::path::Path::new(path)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get directory path"))?;
        tokio::fs::create_dir_all(dir_path).await?;

        // Initialise central variables of async process
        let file = File::create(path).await?;
        let file = Arc::new(Mutex::new(file));
        let num_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let mut tasks = Vec::with_capacity(num_chunks as usize);

        // Build each async task and push to tasks
        for i in 0..num_chunks {
            let start = i * CHUNK_SIZE;
            let end = std::cmp::min((i + 1) * CHUNK_SIZE, file_size);
            let client = self.clone();
            let url = url.to_string();
            let file = file.clone();
            let pb = pb.clone();
            tasks.push(tokio::spawn(async move {
                let mut file = file.lock().await;
                let _download_chunk = client.download_chunk(&url, start, end, &mut file).await;
                pb.inc(end - start);
            }));
        }

        // Join all async tasks together, in order to execute
        let mut outputs = Vec::with_capacity(tasks.len());
        for task in tasks {
            outputs.push(task.await.unwrap());
            sleep(std::time::Duration::from_secs(1)).await;
        }

        // Finish the progress bar
        let file = Arc::try_unwrap(file).unwrap().into_inner();
        let msg = format!(
            "{} downloaded succesfully ({})",
            path,
            indicatif::HumanBytes(file.metadata().await?.len())
        );
        let pb = Arc::try_unwrap(pb).unwrap();
        pb.finish_with_message(msg);

        Ok(())
    }

    /// Download a range of bytes (a chunk) with a GET request.
    async fn download_chunk(
        &self,
        url: &str,
        start: u64,
        end: u64,
        output_file: &mut File,
    ) -> Result<()> {
        let client = self;
        let url = url.to_string();
        let range = format!("bytes={}-{}", start, end - 1);

        // download a range of bytes
        let response = client
            .get(url)
            .header(reqwest::header::RANGE, range)
            .send()
            .await?;

        // seek the position of bytes and write to the file
        let body = response.bytes().await?;
        let _seek = output_file.seek(tokio::io::SeekFrom::Start(start)).await?;
        let _write = output_file.write_all(&body).await?;
        Ok(())
    }
}
