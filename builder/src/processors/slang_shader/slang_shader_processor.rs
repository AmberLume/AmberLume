use std::env::temp_dir;
use std::fs::{read, read_to_string, remove_file};
use std::mem::take;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use blake3::hash;
use crate::processors::processor::Processor;
use crate::build_task::{BuildTask, ShaderTask};
use anyhow::{bail, Result};
use tracing::info;
use crate::cache::{is_dependency_valid, Cache, DependencyRecord};
use crate::dispatcher::Dispatcher;
use crate::processors::utils::resource_key;

pub struct SlangShaderProcessor {
    cache: Arc<Cache>,
}

impl SlangShaderProcessor {
    pub fn create(
        cache: Arc<Cache>,
    ) -> Self {
        Self {
            cache,
        }
    }

    fn call_slangc_for(entry: &Path, dependency_file: &Path) -> Result<Vec<u8>> {
        let params = vec![
            "-target", "spirv",
            "-profile", "spirv_1_6",
            "-matrix-layout-column-major",
            "-O2",
            "-warnings-disable", "39001",
            "-depfile", dependency_file.to_str().unwrap(),
            "-lang", "slang",
            entry.to_str().unwrap(),
        ];

        let output = Command::new("slangc")
            .args(&params)
            .stderr(Stdio::inherit())
            .output()?;

        if !output.status.success() {
            bail!("slangc failed with exit status {}", output.status);
        }

        Ok(output.stdout)
    }

    fn read_dependencies(entry: &Path, dependency_file: &Path) -> Result<Vec<DependencyRecord>> {
        let content = read_to_string(dependency_file)?;
        remove_file(dependency_file)?;

        let Some((_, sources)) = content.split_once(": ") else {
            bail!("Malformed slangc dependency file: {}", content);
        };

        let entry = entry.canonicalize()?;

        let mut dependencies = Vec::new();

        for source in Self::split_sources(sources) {
            let path = Path::new(&source).canonicalize()?;

            if path == entry {
                continue;
            }

            dependencies.push(DependencyRecord {
                path: path.to_string_lossy().into_owned(),
                hash: hash(&read(&path)?).into(),
            });
        }

        dependencies.sort_by(|a, b| a.path.cmp(&b.path));
        dependencies.dedup_by(|a, b| a.path == b.path);

        Ok(dependencies)
    }

    fn split_sources(sources: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut path = String::new();
        let mut characters = sources.trim_end().chars();

        while let Some(character) = characters.next() {
            match character {
                '\\' | '$' => path.extend(characters.next()),
                ' ' => paths.push(take(&mut path)),
                _ => path.push(character),
            }
        }

        paths.push(path);

        paths
    }
}

impl Processor<ShaderTask> for SlangShaderProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ShaderTask) -> Result<()> {
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

        info!("Compiling shader {:?}", task.build_target.relative_full());

        let dependency_file = temp_dir().join(format!("{}.d", hash(key.as_bytes()).to_hex()));

        let artifact = Self::call_slangc_for(entry, &dependency_file)?;
        let dependencies = Self::read_dependencies(entry, &dependency_file)?;

        self.cache.touch(&key, entry_hash, dependencies);
        self.cache.clear_outputs(&key);

        let resource_key = resource_key(&task.build_target, &task.build_target.name, "spv");
        dispatcher.dispatch(BuildTask::archive(
            &task.build_target,
            &resource_key,
            artifact,
        ));

        Ok(())
    }
}
