use std::fs::create_dir_all;
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
    pub fn new(
        source_assets: &Path,
    ) -> Result<Self> {
        let shader_adapter = ShaderAdapter::create(&source_assets);

        Ok(Self { shader_adapter })
    }
}

impl ResourcePipeline for ShaderPipeline {
    fn can_assemble(&self, extension: &str) -> bool {
        ["vert", "frag", "comp"].contains(&extension)
    }

    fn assemble(&mut self, source_path: &Path, generated_root_path: &Path, local_path: &Path) -> Result<()> {
        let name = get_name(&source_path)?;
        let extension = get_extension(&source_path)?;
        let result_path = generated_root_path.join(local_path).join(format!("{}.{}.spv", name, extension));

        let source = read_bytes(source_path)?;

        let compilation_result = self.shader_adapter.adapt(&ShaderResource {
            name: local_path.to_str().unwrap().to_string(),
            extension,
            source_code: String::from_utf8(source)?,
        })?;

        create_dir_all(result_path.parent().unwrap())?;

        write_bytes(&result_path, &compilation_result)?;

        Ok(())
    }
}
