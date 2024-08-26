use super::de_cik;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use rayon::prelude::*;
use serde::Deserializer;
use std::sync::Mutex;


pub async fn fetch(cik_str: &str) -> Result<Vec<CoreCell>> {
    let path = format!("./buffer/companyfacts/CIK{}.json", cik_str);
    let out: SecCompany = renai_common::fs::read_json(&path).await?;
    Ok(out.facts)
}

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

fn de_facts<'de, D>(deserializer: D) -> Result<Vec<CoreCell>, D::Error>
where
    D: Deserializer<'de>,
{
    type Facts = BTreeMap<String, Option<BTreeMap<String, Metric>>>;
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

pub fn de_dates<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use chrono::{Datelike, Duration, NaiveDate, Weekday};
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
