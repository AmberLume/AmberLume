use alpaca::unpacker::asset_data::AssetData;
use amber_lume::platform_providers::io_provider::IOProvider;
use ndk::asset::{Asset, AssetManager};
use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::path::{Path, PathBuf};

pub struct AndroidIOProvider {
    asset_manager: AssetManager,
}

impl AndroidIOProvider {
    pub fn new(asset_manager: AssetManager) -> Self {
        Self { asset_manager }
    }
}

impl IOProvider for AndroidIOProvider {
    fn list_files(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("scenes.alpaca"),
            PathBuf::from("meshes.alpaca"),
            PathBuf::from("materials.alpaca"),
            PathBuf::from("textures.alpaca"),
            PathBuf::from("animations.alpaca"),
            PathBuf::from("skeletons.alpaca"),
            PathBuf::from("physical_bodies.alpaca"),
            PathBuf::from("shaders.alpaca"),
        ]
    }

    fn open(&self, path: &Path) -> Result<Box<dyn AssetData>> {
        let name = path
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 path: {}", path.display()))?;
        let cname = CString::new(name).context("path contains null byte")?;

        let mut asset = self
            .asset_manager
            .open(&cname)
            .ok_or_else(|| anyhow!("asset not found: {name}"))?;

        let buffer: &[u8] = asset.buffer().map_err(|e| anyhow!("buffer: {e:?}"))?;
        let buffer_ptr = buffer.as_ptr();
        let buffer_len = buffer.len();

        Ok(Box::new(AndroidAsset {
            _asset: asset,
            buffer_ptr,
            buffer_len,
        }))
    }
}

unsafe impl Send for AndroidIOProvider {}
unsafe impl Sync for AndroidIOProvider {}

struct AndroidAsset {
    buffer_ptr: *const u8,
    buffer_len: usize,
    _asset: Asset,
}

impl AssetData for AndroidAsset {
    fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.buffer_ptr, self.buffer_len) }
    }
}

unsafe impl Send for AndroidAsset {}
unsafe impl Sync for AndroidAsset {}
