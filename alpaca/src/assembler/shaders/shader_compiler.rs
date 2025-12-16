use crate::assembler::resource_compiler::ResourceCompiler;
use anyhow::Result;
use shaderc::{CompileOptions, Compiler, EnvVersion, OptimizationLevel, ShaderKind, TargetEnv};

pub struct ShaderCompiler {
    handle: Compiler,
    compile_options: CompileOptions<'static>,
}

impl ShaderCompiler {
    pub fn new() -> Result<Self> {
        let compiler = Compiler::new().expect("Could not create shaders assembler");
        let mut compile_options =
            CompileOptions::new().expect("Could not create shaders compile options");

        compile_options.set_optimization_level(OptimizationLevel::Performance);
        compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);

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

pub struct ShaderResource {
    pub name: String,
    pub extension: String,
    pub source_code: String,
}

impl ResourceCompiler for ShaderCompiler {
    type Input = ShaderResource;

    fn compile(&self, input: Self::Input) -> Result<Vec<u8>> {
        let kind = ShaderCompiler::get_kind_of(&input.extension);

        let artifact = self
            .handle
            .compile_into_spirv(
                &input.source_code,
                kind,
                &input.name,
                "main",
                Some(&self.compile_options),
            )
            .unwrap_or_else(|err| panic!("Could not compile shaders: {}", err));

        Ok(artifact.as_binary_u8().to_vec())
    }
}
