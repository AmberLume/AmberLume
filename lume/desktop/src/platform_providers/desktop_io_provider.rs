use resource_reader::AssetBytes;
use resource_reader::IOProvider;
use anyhow::Context;
use anyhow::Result;
use memmap2::Mmap;
use std::env::current_dir;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct MmapAsset {
    mmap: Mmap,
}

impl AssetBytes for MmapAsset {
    fn bytes(&self) -> &[u8] {
        &self.mmap
    }
}

pub struct DesktopIOProvider {
    root: PathBuf,
}

impl DesktopIOProvider {
    pub fn new() -> Self {
        let root = current_dir().unwrap().join("assets");

        Self { root }
    }
}

impl IOProvider for DesktopIOProvider {
    fn list_files(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    fn open(&self, path: &Path) -> Result<Box<dyn AssetBytes>> {
        let file = File::open(path).with_context(|| format!("open {:?}", path))?;
        let mmap = unsafe { Mmap::map(&file) }.with_context(|| format!("mmap {:?}", path))?;

        Ok(Box::new(MmapAsset { mmap }))
    }
}
