use crate::compiler::pipeline::CompilationResult;
use anyhow::Result;
use std::path::Path;

pub trait ResourceCompiler {
    fn extensions(&self) -> &[&str];

    fn compile(&self, name: &String, src: &Path, dst_dir: &Path) -> Result<Vec<CompilationResult>>;
}
