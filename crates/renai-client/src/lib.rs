pub mod client_ext;

pub mod prelude {
    pub use crate::client_ext::couchdb::ClientCouchExt as CouchDB;
    pub use crate::client_ext::util::ClientUtilExt as Util;
    #[allow(unused_imports)]
    pub use crate::client_ext::Client;

    pub use crate::doc;

    pub fn build_client(user_agent: &String) -> anyhow::Result<Client> {
        let client = reqwest::ClientBuilder::new()
            .user_agent(user_agent)
            .build()?;
        Ok(client)
    }
}
