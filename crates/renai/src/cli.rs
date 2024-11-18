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

    /// Clean up directories of the file store.
    Rm {
        directories: Vec<RmArgs>,
    },

    /// Insert datasets into PostgreSQL
    Insert {
        datasets: Vec<Dataset>,
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
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dataset {
    StockIndex,
    StockPrices,
    StockMetrics, // <-- bulk .zip required
    CryptoIndex,
    CryptoPrices,
    // Econ,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RmArgs {
    /// Remove the buffer directory; used in holding bulk data.
    Buffer,
}
