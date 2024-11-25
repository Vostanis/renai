use anyhow::Result;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use tracing::{debug, error, trace};
use zip::ZipArchive;

/// Reads a `.json` file from `path`.
///
/// ```rust
/// let ouput: DesiredType = renai_client::read_json(path).await?;
/// ```
pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = tokio::fs::read(path).await?;
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

/// Unzip a `.zip` file (`zip_file`) to a target directory (`to_dir`).
///
/// `std::fs::create_dir_all(to_dir)?` is used in creating `to_dir` path,
/// so directories will be created, as necessary, by the unzip() function.
pub async fn unzip(zip_file: &str, to_dir: &str) -> anyhow::Result<()> {
    debug!("unzipping {zip_file} to {to_dir}");

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
