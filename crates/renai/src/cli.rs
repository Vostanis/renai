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

    /// Migrate schemas from the CouchDB filestore to the PostgreSQL server.
    Migrate {
        schema: Vec<MigrationArgs>,

        /// Reset the PostgreSQL tables, recreating them from scratch.
        #[arg(short, long)]
        reset: bool,
    },

    /// Clean up directories of the file store.
    Rm {
        directories: Vec<RmArgs>,
    },

    Test,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum FetchArgs {
    /// Run all processes
    All,

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

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum MigrationArgs {
    /// The stock schema (Yahoo! Finance & SEC).
    Stocks,
}
