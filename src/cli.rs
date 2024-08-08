use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Retrieve data from endpoints, specifying which steps of the process to enact.
    Fetch {
        actions: Vec<FetchArgs>,
    },

    /// Shortcut to retrieve all data from endpoints, running every step.
    FetchAll,

    /// Clean up directories of the file store.
    Rm {
        directories: Vec<RmArgs>,
    },

    Test,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum FetchArgs {
    /// Get the bulk zip data file.
    Bulk,

    /// Unzip the SEC bulk zip file.
    Unzip,

    /// Collect price & core data, and upload it.
    Collection,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RmArgs {
    /// Remove the buffer directory; used in holding bulk data.
    Buffer,
}
