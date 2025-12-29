use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;

pub fn for_each_file<F>(from: impl AsRef<Path>, mut callback: F) -> Result<()>
where
    F: FnMut(&Path) -> Result<()>,
{
    for entry in WalkDir::new(&from).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(&from)?;

            callback(relative_path)?;
        }
    }

    Ok(())
}

pub fn get_name(path: &Path) -> Result<String> {
    Ok(path
        .file_stem()
        .context(format!("Failed to get file name: {}", path.display()))?
        .to_str()
        .unwrap()
        .to_owned())
}

pub fn get_extension(path: &Path) -> Result<String> {
    Ok(path
        .extension()
        .context(format!("Failed to get file extension: {}", path.display()))?
        .to_str()
        .unwrap()
        .to_owned())
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>> {
    let mut source_file = File::open(path)?;
    let mut data = Vec::new();
    source_file.read_to_end(&mut data)?;

    Ok(data)
}

pub fn write_bytes(path: &Path, data: &[u8]) -> Result<()> {
    let mut result_file = File::create(path)?;
    result_file.write(&data)?;

    Ok(())
}
