use crate::assembler::resource_compiler::ResourceCompiler;
use crate::assembler::resource_pipeline::ResourcePipeline;
use crate::assembler::shaders::shader_compiler::{ShaderCompiler, ShaderResource};
use crate::assembler::utils::{get_extension, get_name, read_bytes, write_bytes};
use anyhow::Result;
use std::path::Path;

pub struct ShaderPipeline {
    shader_compiler: ShaderCompiler,
}

impl ShaderPipeline {
    pub fn new() -> Result<Self> {
        let shader_compiler = ShaderCompiler::new()?;

        Ok(Self { shader_compiler })
    }
}

impl ResourcePipeline for ShaderPipeline {
    fn can_assemble(&self, extension: &str) -> bool {
        ["vert", "frag"].contains(&extension)
    }

    fn assemble(&self, source_path: &Path, target_path: &Path) -> Result<()> {
        let name = get_name(&source_path)?;
        let extension = get_extension(&source_path)?;
        let result_path = target_path.parent().unwrap().join(format!("{}.spv", &name));

        let source = read_bytes(source_path)?;

        let compilation_result = self.shader_compiler.compile(ShaderResource {
            name,
            extension,
            source_code: String::from_utf8(source)?,
        })?;

        write_bytes(&result_path, &compilation_result)?;

        Ok(())
    }
}
