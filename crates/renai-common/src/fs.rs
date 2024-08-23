use anyhow::Result;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;
use zip::ZipArchive;

pub async fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = tokio::fs::read(path).await?;
    let data: T = serde_json::from_slice(&file)?;
    Ok(data)
}

pub async fn unzip(zip_file: &str, to_dir: &str) -> Result<()> {
    let file = std::fs::File::open(zip_file)?;
    let archive = ZipArchive::new(file)?;
    let zip_length = archive.len();
    let archive = Arc::new(std::sync::Mutex::new(archive));
    let pb = crate::tui::single_pb(zip_length as u64);

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