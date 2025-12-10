use anyhow::Result;
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
