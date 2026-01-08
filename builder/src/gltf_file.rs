use std::fs::File;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use gltf::Document;
use gltf::json::Root;
use memmap2::Mmap;

pub struct GltfFile {
    mmap: Arc<Mmap>,
    json_range: Range<usize>,
    bin_mmap: Option<Arc<Mmap>>,
}

impl GltfFile {
    pub fn create(path: &Path) -> anyhow::Result<GltfFile> {
        let file = File::open(path)?;
        let mmap = Arc::new(unsafe { Mmap::map(&file) }?);

        let json_range = 0..mmap.len();

        let bin_path = path.with_extension("bin");
        let bin_mmap = if bin_path.exists() {
            let bin_file = File::open(&bin_path)?;
            Some(Arc::new(unsafe { Mmap::map(&bin_file) }?))
        } else {
            None
        };

        Ok(GltfFile {
            mmap,
            json_range,
            bin_mmap,
        })
    }

    pub fn bin(&self) -> Option<&[u8]> {
        if let Some(bin_mmap) = self.bin_mmap.as_ref() {
            Some(bin_mmap.as_ref())
        } else {
            None
        }
    }

    pub fn get_document(&self) -> anyhow::Result<Document> {
        let json_slice = &self.mmap[self.json_range.clone()];

        let root = Root::from_slice(json_slice)?;
        let document = Document::from_json(root)?;

        Ok(document)
    }
}
