use rayon::iter::{IntoParallelIterator, ParallelIterator};
use renai_client::client_ext::util::ClientUtilExt;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, error, trace};
use zip::ZipArchive;

/// Download large files;
///     1. companyfacts.zip ~ 1.1GB
///     2. submissions.zip ~ 1.3GB
pub async fn download_zip_file(http_client: &reqwest::Client) -> anyhow::Result<()> {
    // 1. companyfacts.zip
    // let url = "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip";
    let path = "./buffer/companyfacts.zip";
    // debug!("downloading companyfacts.zip");
    // http_client.download_file(url, path).await?;
    // debug!("downloaded companyfacts.zip");

    debug!("unzipping companyfacts.zip");
    unzip(path, "./buffer/companyfacts").await?;
    debug!("companyfacts.zip unzipped successfully");

    // 2. submissions.zip
    // let url = "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip";
    let path = "./buffer/submissions.zip";
    // debug!("downloading submissions.zip");
    // http_client.download_file(url, path).await?;
    // debug!("downloaded submissions.zip");

    debug!("unzipping submissions.zip");
    unzip(path, "./buffer/submissions").await?;
    debug!("submissions.zip unzipped successfully");

    Ok(())
}

/// Reads a `.json` file from `path`.
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> anyhow::Result<T> {
    trace!("reading file at path: {}", path);
    let file = fs::read(path).await?;
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

/// Unzip a `.zip` file (`zip_file`) to a target directory (`to_dir`).
///
/// `std::fs::create_dir_all(to_dir)?` is used in creating `to_dir` path,
/// so directories will be created, as necessary, by the unzip() function.
pub async fn unzip(zip_file: &str, to_dir: &str) -> anyhow::Result<()> {
    // Use of rayon requires lots of async wrappings
    let file = std::fs::File::open(zip_file)?;
    let archive = ZipArchive::new(file).map_err(|e| {
        error!("failed to open zip file at {}, {}", zip_file, e);
        e
    })?;
    let zip_length = archive.len();
    let archive = Arc::new(std::sync::Mutex::new(archive));

    // Ensure the target directory exists
    std::fs::create_dir_all(to_dir)?;

    // Parallel iteration across zipped files
    (0..zip_length).into_par_iter().for_each(|i| {
        let archive = archive.clone();
        let mut archive = archive.lock().expect("unlock zip archive");
        let mut file = archive.by_index(i).expect("file from zip archive");
        let outpath = format!("{to_dir}/{}", file.mangled_name().display());
        let outdir = std::path::Path::new(&outpath)
            .parent()
            .expect("parent directory of output path");
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir).expect("failed to create directory");
        }

        // Extract the file
        let mut outfile = std::fs::File::create(&outpath).expect("creation of output file");
        trace!("copying {} to {}", file.name(), outpath);
        std::io::copy(&mut file, &mut outfile).expect("copying of zip file to output");
    });

    trace!("{zip_file} unzipped to {to_dir}");

    Ok(())
}
