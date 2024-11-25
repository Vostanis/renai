use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Sets the level of tracing
    #[arg(long, default_value = "INFO")]
    pub trace: TraceLevel,
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
    /// Get the bulk zip data file.
    Bulks,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dataset {
    StockIndex,
    StockPrices,
    StockMetrics, // <-- bulk .zip required
    CryptoIndex,
    CryptoPrices,
    Econ,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum RmArgs {
    /// Remove the buffer directory; used in holding bulk data.
    Buffer,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum TraceLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}
