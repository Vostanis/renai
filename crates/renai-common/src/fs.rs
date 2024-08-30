use anyhow::Result;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
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
/// `std::fs::create_dir_all(to_dir)?` is used in creating `to_dir` path,
/// so directories will be created, as necessary, by the unzip() function.
pub async fn unzip(zip_file: &str, to_dir: &str) -> Result<()> {
    // Use of rayon requires lots of async wrappings
    let file = std::fs::File::open(zip_file)?;
    let archive = ZipArchive::new(file)?;
    let zip_length = archive.len();
    let archive = Arc::new(std::sync::Mutex::new(archive));
    let pb = crate::tui::single_pb(zip_length as u64);

    // Ensure the target directory exists
    std::fs::create_dir_all(to_dir)?;

    // Parallel iteration across zipped files
    (0..zip_length).into_par_iter().for_each(|i| {
        let archive = archive.clone();
        let mut archive = archive.lock().unwrap();
        let mut file = archive.by_index(i).unwrap();
        let outpath = format!("{to_dir}/{}", file.mangled_name().display());
        let outdir = std::path::Path::new(&outpath).parent().unwrap();
        if !outdir.exists() {
            std::fs::create_dir_all(&outdir).unwrap();
        }

        // Extract the file
        let mut outfile = std::fs::File::create(&outpath).unwrap();
        std::io::copy(&mut file, &mut outfile).unwrap();
        pb.inc(1);
    });

    let msg = format!("{zip_file} unzipped to {to_dir}");
    pb.finish_with_message(msg);
    Ok(())
}
