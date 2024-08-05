use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Retrieve data from endpoints, specifying which steps of the process to enact.
    Fetch(FetchArgs),

    /// Shortcut to retrieve all data from endpoints, running every step.
    FetchAll,

    /// Clean up directories of the file store.
    Rm(RmArgs),

    /// Upload data to the database.
    Upload(UploadArgs),
}

#[derive(Args, Debug)]
pub struct FetchArgs {
    /// Get the bulk zip data file.
    pub bulk: Option<String>,

    /// Unzip the SEC bulk zip file.
    pub unzip: Option<String>,
}

#[derive(Args, Debug)]
pub struct RmArgs {
    /// Remove the buffer directory, used in holding bulk data.
    pub buffer: Option<String>,
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    /// Upload all the price & core stock data.
    pub all: Option<String>,
}
