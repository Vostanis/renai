pub mod client_ext;

pub mod prelude {
    #[allow(unused_imports)]
    pub use crate::client_ext::Client;
    pub use crate::client_ext::couchdb::ClientCouchExt as CouchDB;
    pub use crate::client_ext::util::ClientUtilExt as Util;

    pub fn build_client() -> anyhow::Result<Client> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(&std::env::var("USER_AGENT")?)
            .build()?;
        Ok(client)
    }
}