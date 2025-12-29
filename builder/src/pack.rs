use crate::utils::for_each_file;
use alpaca::packer::alpaca_writer::AlpacaWriter;
use anyhow::Result;
use std::fs::{create_dir_all, read};
use std::path::PathBuf;

pub fn pack_all(
    pack_name: &str,
    align: u64,
    source: &PathBuf,
    target: &PathBuf,
) -> Result<()> {
    println!("Packing to Alpaca: {}", &source.display());

    create_dir_all(&target)?;

    let mut alpaca_writer = AlpacaWriter::create(pack_name.to_owned(), target.clone(), align)?;

    for_each_file(&source, |path| {
        let relative_path = path.to_string_lossy().into_owned();
        let source = source.join(path);

        println!("Packing {}...", relative_path);

        let data = read(source)?;

        alpaca_writer.push(&relative_path, &data)?;

        Ok(())
    })?;

    alpaca_writer.pack()?;

    Ok(())
}
