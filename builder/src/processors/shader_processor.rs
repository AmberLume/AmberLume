use std::fs::{read, read_to_string};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use blake3::hash;
use parking_lot::Mutex;
use shaderc::{CompileOptions, Compiler, EnvVersion, OptimizationLevel, ResolvedInclude, ShaderKind, SpirvVersion, TargetEnv};
use crate::processors::processor::Processor;
use crate::build_task::{BuildTask, ShaderTask};
use anyhow::Result;
use tracing::info;
use crate::cache::{is_dependency_valid, Cache, DependencyRecord};
use crate::dispatcher::Dispatcher;
use crate::processors::utils::resource_key_from_build_target;

pub struct ShaderProcessor {
    compiler: Compiler,

    cache: Arc<Cache>,
}

impl ShaderProcessor {
    pub fn create(
        cache: Arc<Cache>,
    ) -> Self {
        let compiler = Compiler::new()
            .expect("Could not create shaders assembler");

        Self {
            compiler,

            cache,
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

    fn get_options(recorded: Arc<Mutex<Vec<DependencyRecord>>>) -> CompileOptions<'static> {
        let mut compile_options = CompileOptions::new()
            .expect("Could not create shaders compile options");

        compile_options.set_target_spirv(SpirvVersion::V1_6);
        compile_options.set_optimization_level(OptimizationLevel::Performance);
        compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_3 as u32);

        compile_options.add_macro_definition("GL_EXT_buffer_reference", Some("1"));

        compile_options.set_include_callback(move |requested_source, _type, requesting_source, _depth| {
            let mut path = PathBuf::from(requesting_source);
            path.pop();

            let include_path = path.join(requested_source);

            let content = read_to_string(&include_path).expect("Could not read shader include path");

            let resolved_name = include_path.canonicalize()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| include_path.to_string_lossy().into_owned());

            recorded.lock().push(DependencyRecord {
                path: resolved_name.clone(),
                hash: hash(content.as_bytes()).into(),
            });

            Ok(ResolvedInclude {
                resolved_name,
                content,
            })
        });

        compile_options
    }
}

impl Processor<ShaderTask> for ShaderProcessor{
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ShaderTask) -> Result<()> {
        let extension = &task.build_target.extension;
        let entry = &task.build_target.entry;
        let key = entry.to_string_lossy().into_owned();

        let data_raw = read(entry)?;
        let entry_hash: [u8; 32] = hash(&data_raw).into();

        if let Some(node) = self.cache.lookup(&key) {
            let dependencies_are_valid = node.dependencies.iter().all(is_dependency_valid);
            let outputs_are_present = node.outputs.iter().all(|o| Path::new(o).exists());

            if node.hash == entry_hash && dependencies_are_valid && outputs_are_present {
                self.cache.touch(&key, node.hash, node.dependencies);

                info!("Cached shader {:?}", task.build_target.relative_full());

                return Ok(());
            }
        }

        let kind = Self::get_kind_of(&extension);

        info!("Compiling shader {:?}", task.build_target.relative_full());

        let source_code = String::from_utf8_lossy(&data_raw);

        let recorded = Arc::new(Mutex::new(Vec::<DependencyRecord>::new()));
        let options = Self::get_options(recorded.clone());

        let artifact = self
            .compiler
            .compile_into_spirv(
                &source_code,
                kind,
                entry.to_str().unwrap(),
                "main",
                Some(&options),
            )
            .expect("Could not compile shaders");

        let mut dependencies = take(&mut *recorded.lock());
        dependencies.sort_by(|a, b| a.path.cmp(&b.path));
        dependencies.dedup_by(|a, b| a.path == b.path);

        self.cache.touch(&key, entry_hash, dependencies);
        self.cache.clear_outputs(&key);

        let resource_key = resource_key_from_build_target(&task.build_target, "spv");
        dispatcher.dispatch(BuildTask::archive(
            &task.build_target,
            &resource_key,
            artifact.as_binary_u8().to_vec(),
        ));

        Ok(())
    }
}
