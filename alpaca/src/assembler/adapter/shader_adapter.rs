use crate::assembler::adapter::adapter::ResourceAdapter;
use anyhow::Result;
use shaderc::{CompileOptions, Compiler, EnvVersion, OptimizationLevel, ShaderKind, TargetEnv};

pub struct ShaderAdapter {
    handle: Compiler,
    compile_options: CompileOptions<'static>,
}

impl ShaderAdapter {
    pub fn create() -> Self {
        let compiler = Compiler::new().expect("Could not create shaders assembler");
        let mut compile_options =
            CompileOptions::new().expect("Could not create shaders compile options");

        compile_options.set_optimization_level(OptimizationLevel::Performance);
        compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);

        compile_options.add_macro_definition("GL_EXT_buffer_reference", Some("1"));

        Self {
            handle: compiler,
            compile_options,
        }
    }

    fn get_kind_of(extension: &str) -> ShaderKind {
        match extension {
            "vert" => ShaderKind::Vertex,
            "frag" => ShaderKind::Fragment,
            "comp" => ShaderKind::Compute,
            _ => panic!("Unsupported shaders extension: {}", extension),
        }
    }
}

pub struct ShaderResource {
    pub name: String,
    pub extension: String,
    pub source_code: String,
}

impl ResourceAdapter for ShaderAdapter {
    type Input<'a> = ShaderResource;

    type Output = Vec<u8>;

    fn adapt<'a>(&mut self, input: &Self::Input<'a>) -> Result<Self::Output> {
        let kind = Self::get_kind_of(&input.extension);

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
