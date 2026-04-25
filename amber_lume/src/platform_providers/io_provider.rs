use anyhow::Result;
use std::path::{Path, PathBuf};
use alpaca::unpacker::asset_data::AssetData;

pub trait IOProvider: Send + Sync {
    fn list_files(&self) -> Vec<PathBuf>;

    fn open(&self, path: &Path) -> Result<Box<dyn AssetData>>;
}
