use anyhow::Result;
use std::path::Path;

pub trait ResourcePipeline {
    fn can_assemble(&self, extension: &str) -> bool;

    fn assemble(&mut self, source_path: &Path, generated_root_path: &Path, local_path: &Path) -> Result<()>;
}
