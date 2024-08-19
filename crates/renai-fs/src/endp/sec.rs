use super::us_company_index::de_cik;
use crate::ui;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use zip::ZipArchive;

//////////////////////////////////////////////////////////////////////////////////////
// Functions
//////////////////////////////////////////////////////////////////////////////////////

pub async fn extran(cik_str: &str) -> Result<Vec<CoreCell>> {
    let path = format!("./buffer/companyfacts/CIK{}.json", cik_str);
    let out: SecCompany = read_json(&path).await?;
    Ok(out.facts)
}

pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = tokio::fs::read(path).await?;
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

pub async fn unzip() -> Result<()> {
    let file = fs::File::open("./buffer/companyfacts.zip")?;
    let archive = ZipArchive::new(file)?;
    let zip_length = archive.len();
    let archive = Arc::new(std::sync::Mutex::new(archive));
    let pb = ui::single_pb(zip_length as u64);

    // Parallel iteration across zipped files
    (0..zip_length).into_par_iter().for_each(|i| {
        let archive = archive.clone();
        let mut archive = archive.lock().unwrap();
        let mut file = archive.by_index(i).unwrap();
        let outpath = format!("./buffer/companyfacts/{}", file.mangled_name().display());
        let outdir = std::path::Path::new(&outpath).parent().unwrap();
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir).unwrap();
        }

        // Extract the file
        let mut outfile = fs::File::create(&outpath).unwrap();
        std::io::copy(&mut file, &mut outfile).unwrap();
        pb.inc(1);
    });
    pb.finish_with_message("Company Facts unzipped");
    Ok(())
}

//////////////////////////////////////////////////////////////////////////////////////
// Schema
//////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct SecCompany {
    #[serde(deserialize_with = "de_cik")]
    pub cik: String,
    #[serde(rename = "entityName")]
    pub entity_name: String,
    #[serde(deserialize_with = "de_facts")]
    pub facts: Vec<CoreCell>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CoreCell {
    pub dated: String,
    #[serde(flatten)]
    pub metrics: BTreeMap<String, f64>,
}

use rayon::prelude::*;
use serde::Deserializer;
use std::sync::Mutex;
pub type Facts = BTreeMap<String, Option<BTreeMap<String, Metric>>>;
fn de_facts<'de, D>(deserializer: D) -> Result<Vec<CoreCell>, D::Error>
where
    D: Deserializer<'de>,
{
    let input: Facts = Deserialize::deserialize(deserializer)?;
    let btree: Arc<Mutex<BTreeMap<String, BTreeMap<String, f64>>>> =
        Arc::new(Mutex::new(BTreeMap::new()));

    for (key, value) in input {
        match key.as_str() {
            // the only 2 datasets I'm aware of or intereseted in
            "dei" | "us-gaap" => {
                if let Some(metric_dataset) = value {
                    let btree = Arc::clone(&btree);
                    metric_dataset
                        .into_par_iter()
                        .for_each(|(metric_name, metric_data)| {
                            let mut btree = btree.lock().unwrap();
                            for (_unit, data) in metric_data.units {
                                for val in data {
                                    if let Some(already) =
                                        btree.get_mut(&val.end_date.clone().unwrap())
                                    {
                                        already.insert(metric_name.clone(), val.val.unwrap());
                                    } else {
                                        btree.insert(
                                            val.end_date.clone().unwrap(),
                                            BTreeMap::from([(
                                                metric_name.clone(),
                                                val.val.unwrap(),
                                            )]),
                                        );
                                    }
                                }
                            }
                        });
                };
            }
            // everything else is just ignored
            _ => {}
        }
    }

    let unlocked_btree = Arc::try_unwrap(btree)
        .map_err(|_| serde::de::Error::custom("Failed to unlock data"))?
        .into_inner()
        .map_err(|_| serde::de::Error::custom("Failed to get data"))?;

    let set: Vec<CoreCell> = unlocked_btree
        .into_iter()
        .map(|(date, metrics)| CoreCell {
            dated: date,
            metrics,
        })
        .collect();

    Ok(set)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metric {
    // pub label: Option<String>,
    // pub description: Option<String>,
    pub units: BTreeMap<String, Vec<Values>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Values {
    // #[serde(rename = "start")] // renamed to match "end" keyword issue below
    // pub start_date: Option<String>,
    #[serde(rename = "end", deserialize_with = "de_dates")] // "end" keyword in PostgreSQL
    pub end_date: Option<String>,
    pub val: Option<f64>,
    // pub accn: Option<String>,
    // pub fy: Option<i32>,
    // pub fp: Option<String>,
    // pub form: Option<String>,
    // pub filed: Option<String>,
    // pub frame: Option<String>,
}

use chrono::{Datelike, Duration, NaiveDate, Weekday};
pub fn de_dates<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let de: String = String::deserialize(deserializer)?;
    let date = NaiveDate::parse_from_str(&de, "%Y-%m-%d").unwrap();
    let de = match date.weekday() {
        Weekday::Sat => date + Duration::days(2),
        Weekday::Sun => date + Duration::days(1),
        _ => date,
    };
    let de = de.to_string();
    Ok(Some(de))
}
