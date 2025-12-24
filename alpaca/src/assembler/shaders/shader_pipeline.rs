use crate::assembler::adapter::adapter::ResourceAdapter;
use crate::assembler::adapter::shader_adapter::{ShaderAdapter, ShaderResource};
use crate::assembler::resource_pipeline::ResourcePipeline;
use crate::assembler::utils::{get_extension, get_name, read_bytes, write_bytes};
use anyhow::Result;
use std::path::Path;

pub struct ShaderPipeline {
    shader_adapter: ShaderAdapter,
}

impl ShaderPipeline {
    pub fn new() -> Result<Self> {
        let shader_adapter = ShaderAdapter::create();

        Ok(Self { shader_adapter })
    }
}

impl ResourcePipeline for ShaderPipeline {
    fn can_assemble(&self, extension: &str) -> bool {
        ["vert", "frag"].contains(&extension)
    }

    fn assemble(&mut self, source_path: &Path, target_path: &Path) -> Result<()> {
        let name = get_name(&source_path)?;
        let extension = get_extension(&source_path)?;
        let result_path = target_path.parent().unwrap().join(format!("{}.spv", &name));

        let source = read_bytes(source_path)?;

        let compilation_result = self.shader_adapter.adapt(&ShaderResource {
            name,
            extension,
            source_code: String::from_utf8(source)?,
        })?;

        write_bytes(&result_path, &compilation_result)?;

        Ok(())
    }
}
