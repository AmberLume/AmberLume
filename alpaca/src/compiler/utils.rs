use anyhow::Result;
use std::fs::{create_dir_all, write};
use std::path::Path;

pub fn extension_of(path: &Path) -> String {
    path.extension().unwrap().to_string_lossy().to_string()
}

pub fn write_slice(dst_dir: &Path, name: &String, data: &[u8]) -> Result<()> {
    let file_path = dst_dir.join(&name);

    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }

    write(&file_path, data)?;

    Ok(())
}
