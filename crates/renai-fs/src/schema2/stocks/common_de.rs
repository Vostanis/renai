use chrono::DateTime;
use serde::{Deserialize, Deserializer};

/// Used within the SEC datasets; each company is given a CIK code (and ticker, and title),
/// intended to be a 10-character string, as below:
/// 
///     0000004321 - NVDA - Nvidia
/// 
/// But, many encounter the common issue, as below:
/// 
///     4321 - NVDA - Nvidia
/// 
/// `de_cik` is designed to handle both.
pub fn de_cik<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // general deserialisation, followed by match statement (depending on type found)
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        // if it's a num, pad it as a 10-char string
        serde_json::Value::Number(num) => {
            if let Some(i32_value) = num.as_i64() {
                // as_i64() does the same job for i32
                Ok(format!("{:010}", i32_value))
            } else {
                Err(serde::de::Error::custom(
                    "ERROR! Unable to parse i32 from JSON",
                ))
            }
        }

        // if it's a string, then Ok()
        serde_json::Value::String(s) => Ok(s),

        // else return an error (it can't be correct type)
        _ => Err(serde::de::Error::custom("ERROR! Invalid type for CIK")),
    }
}

/// Transform a `unix timestamp`    -> `naive date`, e.g.,
///             `1705795200`        -> `2024-01-01`
pub fn de_timestamps_to_naive_date<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamps: Vec<i64> = Deserialize::deserialize(deserializer)?;
    let dates = timestamps
        .into_iter()
        .map(|timestamp| {
            DateTime::from_timestamp(timestamp, 0)
                .expect("Expected Vector of Timestamp integers")
                .date_naive()
                .to_string()
        })
        .collect();
    Ok(dates)
}