use crate::asset_bytes::AssetBytes;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub trait IOProvider: Send + Sync {
    fn list_files(&self) -> Vec<PathBuf>;

    fn open(&self, path: &Path) -> Result<Box<dyn AssetBytes>>;
}
