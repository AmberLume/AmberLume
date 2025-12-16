use anyhow::Result;
use std::path::Path;

pub trait ResourcePipeline {
    fn can_assemble(&self, extension: &str) -> bool;

    fn assemble(&self, source_path: &Path, target_path: &Path) -> Result<()>;
}
