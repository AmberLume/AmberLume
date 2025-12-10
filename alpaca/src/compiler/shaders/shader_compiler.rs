use crate::compiler::pipeline::CompilationResult;
use crate::compiler::resource_compiler::ResourceCompiler;
use crate::compiler::utils::{extension_of, write_slice};
use anyhow::Result;
use shaderc::{CompileOptions, Compiler, OptimizationLevel, ShaderKind, TargetEnv};
use std::fs::read_to_string;
use std::path::Path;

pub struct ShaderCompiler {
    handle: Compiler,
    compile_options: CompileOptions<'static>,
}

impl ShaderCompiler {
    pub fn new() -> Result<Self> {
        let compiler = Compiler::new().expect("Could not create shaders compiler");
        let mut compile_options =
            CompileOptions::new().expect("Could not create shaders compile options");

        compile_options.set_optimization_level(OptimizationLevel::Performance);
        compile_options.set_target_env(TargetEnv::Vulkan, 0);

        Ok(Self {
            handle: compiler,
            compile_options,
        })
    }

    pub fn get_kind_of(extension: &str) -> ShaderKind {
        match extension {
            "vert" => ShaderKind::Vertex,
            "frag" => ShaderKind::Fragment,
            _ => panic!("Unsupported shaders extension: {}", extension),
        }
    }
}

impl<'compiler> ResourceCompiler for ShaderCompiler {
    fn extensions(&self) -> &[&str] {
        &["vert", "frag"]
    }

    fn compile(&self, name: &String, src: &Path, dst_dir: &Path) -> Result<Vec<CompilationResult>> {
        let kind = Self::get_kind_of(&extension_of(src));
        let data = read_to_string(src)?;

        let artifact = self
            .handle
            .compile_into_spirv(&data, kind, &name, "main", Some(&self.compile_options))
            .unwrap_or_else(|err| panic!("Could not compile shaders: {}", err));

        let name = format!("{}.spv", name);
        let data = artifact.as_binary_u8().to_vec();

        write_slice(dst_dir, &name, &data)?;

        let result = CompilationResult { name };

        Ok(vec![result])
    }
}
