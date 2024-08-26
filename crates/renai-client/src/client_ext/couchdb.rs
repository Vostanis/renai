// use crate::client_ext::
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::Future;

///  CouchDB Document definition.
#[derive(Deserialize, Serialize, Debug)]
pub struct Document<T> {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _rev: String,
    pub data: T,
}

/// Shortcut macro for defining CouchDB Documents.
#[macro_export]
macro_rules! doc {
    ($doc_id:expr, $data:expr) => {
        renai_client::client_ext::couchdb::Document {
            _id: $doc_id.to_string(),
            _rev: "".to_string(),
            data: $data,
        }
    };
}

/// Used in (de)serializing document transfers in the
/// CouchDB protocol; see [`insert_doc()`] for more.
///
/// [`insert_doc()`]: ./trait.ClientCouchExt.html#method.insert_doc
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DocumentMeta {
    _id: String,
    _rev: String,
}

pub trait ClientCouchExt {
    fn insert_doc<T>(
        &self,
        data: &T,
        conn: &str,
        doc_id: &str,
    ) -> impl Future<Output = Result<()>> + Send
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Sync;
}

impl ClientCouchExt for Client { 
    async fn insert_doc<T>(
        &self, 
        data: &T, 
        conn: &str, 
        doc_id: &str
    ) -> Result<()>
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
                let current_file: DocumentMeta = serde_json::from_str(&text)
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
}