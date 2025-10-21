use anyhow::Result;
use shaderc::{CompileOptions, Compiler, OptimizationLevel, ShaderKind, TargetEnv};
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct ShaderCompiler<'compiler> {
    handle: Compiler,
    compile_options: CompileOptions<'compiler>,
}

impl<'compiler> ShaderCompiler<'compiler> {
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

    pub fn compile_all(&self, src_dir: &PathBuf, dst_dir: &PathBuf) -> Result<()> {
        let compiler = ShaderCompiler::new()?;

        for entry in WalkDir::new(src_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();

                compiler.compile(path, &dst_dir)?;
            }
        }

        Ok(())
    }

    fn compile(&self, target_file: PathBuf, dst_dir: &PathBuf) -> Result<()> {
        let extension = target_file.extension().and_then(|e| e.to_str()).unwrap();

        let kind = match extension {
            "vert" => ShaderKind::Vertex,
            "frag" => ShaderKind::Fragment,
            _ => panic!("Unsupported shaders extension: {}", extension),
        };

        let source = read_to_string(target_file.clone())?;
        let file_name = target_file
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let artifact = self
            .handle
            .compile_into_spirv(
                &source,
                kind,
                &file_name,
                "main",
                Some(&self.compile_options),
            )
            .unwrap_or_else(|err| panic!("Could not compile shaders: {}", err));

        let output_file_name = format!("{}.spv", file_name);
        let output_file = dst_dir.join(output_file_name);

        create_dir_all(dst_dir)?;

        eprintln!("Creating {}", output_file.display());

        write(output_file, artifact.as_binary_u8())?;

        Ok(())
    }
}
