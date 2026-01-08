use std::fs::{read, read_to_string};
use std::path::PathBuf;
use std::sync::Arc;
use shaderc::{CompileOptions, Compiler, EnvVersion, OptimizationLevel, ShaderKind, SpirvVersion, TargetEnv};
use crate::processors::processor::Processor;
use crate::build_task::{BuildTask, ShaderTask, WriteFileTask};
use anyhow::Result;
use tracing::info;
use crate::dispatcher::Dispatcher;

pub struct ShaderProcessor {
    compiler: Compiler,
}

impl ShaderProcessor {
    pub fn create() -> Self {
        let compiler = Compiler::new()
            .expect("Could not create shaders assembler");

        Self {
            compiler,
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

    fn get_options(&self) -> CompileOptions<'static> {
        let mut compile_options = CompileOptions::new()
            .expect("Could not create shaders compile options");

        compile_options.set_target_spirv(SpirvVersion::V1_5);
        compile_options.set_optimization_level(OptimizationLevel::Performance);
        compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);

        compile_options.add_macro_definition("GL_EXT_buffer_reference", Some("1"));

        compile_options.set_include_callback(move |requested_source, _type, requesting_source, _depth| {
            let mut path = PathBuf::from(requesting_source);
            path.pop();

            let include_path = path.join(requested_source);

            let content = read_to_string(&include_path).expect("Could not read shader include path");

            Ok(shaderc::ResolvedInclude {
                resolved_name: include_path.to_string_lossy().to_string(),
                content,
            })
        });

        compile_options
    }
}

impl Processor<ShaderTask> for ShaderProcessor{
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ShaderTask) -> Result<()> {
        let kind = Self::get_kind_of(&task.paths.extension);

        info!("Compiling shader {}", task.paths.relative.display());

        let data_raw = read(&task.paths.source_file())?;
        let source_code = String::from_utf8_lossy(&data_raw);
        let options = self.get_options();

        let artifact = self
            .compiler
            .compile_into_spirv(
                &source_code,
                kind,
                &task.paths.source_file().to_str().unwrap(),
                "main",
                Some(&options),
            )
            .expect("Could not compile shaders");

        let name = format!("{}.{}.spv", task.paths.name, task.paths.extension);

        dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
            target_path: task.paths.target.join(&name),
            
            data: artifact.as_binary_u8().into(),
        }));
        
        Ok(())
    }
}
