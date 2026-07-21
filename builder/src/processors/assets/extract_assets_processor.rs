use std::fs::{canonicalize, read};
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use blake3::hash;
use gltf::Document;
use gltf::image::Source;
use tracing::info;
use crate::build_task::ExtractAssetsTask;
use crate::cache::{is_dependency_valid, Cache, DependencyRecord};
use crate::dispatcher::Dispatcher;
use crate::processors::assets::adapter::asset_model::AssetModel;
use crate::processors::assets::writer::animation_writer::write_animation_data;
use crate::processors::assets::utils::gltf_file::GltfFile;
use crate::processors::assets::writer::bones_writer::write_bones_data;
use crate::processors::assets::writer::mesh_writer::{write_mesh_data_flat};
use crate::processors::assets::writer::physical_body_writer::write_physical_body_data_flat;
use crate::processors::assets::writer::scene_writer::write_scene_data_flat;
use crate::processors::processor::Processor;

pub struct ExtractAssetsProcessor {
    cache: Arc<Cache>,
}

impl ExtractAssetsProcessor {
    pub fn create(
        cache: Arc<Cache>
    ) -> Self { 
        Self {
            cache,
        } 
    }

    fn extract(
        &self,
        dispatcher: Arc<Dispatcher>,
        task: &ExtractAssetsTask,
        key: &str,
        entry_hash: [u8; 32],
    ) -> Result<()> {
        info!("Parsing GLTF {:?}", task.build_target.relative_full());

        let gltf_file = Arc::new(GltfFile::create(&task.build_target.entry)?);

        let document = gltf_file.get_document()?;

        let dependencies = collect_dependencies(&task.build_target.entry, &document)?;

        self.cache.touch(key, entry_hash, dependencies);
        self.cache.clear_outputs(key);

        let model = AssetModel::from_document(&document, gltf_file.bin())?;

        if !model.is_empty() {
            if !model.placeholders.is_empty() {
                info!("Importing SCENE (flag) {:?}", task.build_target.relative_full());

                write_scene_data_flat(dispatcher.clone(), &task.build_target, model.placeholders)?;
            } else if !model.skeletons.is_empty() {
                info!("Importing SKELETON (flag) {:?}", task.build_target.relative_full());

                for skeleton in model.skeletons {
                    let skeleton_data = write_bones_data(dispatcher.clone(), &task.build_target, skeleton)?;

                    for animation in document.animations() {
                        write_animation_data(dispatcher.clone(), &task.build_target, &skeleton_data, &animation, gltf_file.bin())?;
                    }
                }
            } else {
                if !model.meshes.is_empty() {
                    info!("Importing MESH (flag) {:?}", task.build_target.relative_full());

                    write_mesh_data_flat(dispatcher.clone(), &task.build_target, model.meshes)?;
                }

                write_physical_body_data_flat(dispatcher.clone(), &task.build_target, model.colliders)?;
            }
        }

        Ok(())
    }
}

impl Processor<ExtractAssetsTask> for ExtractAssetsProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ExtractAssetsTask) -> Result<()> {
        let entry = &task.build_target.entry;
        let key = entry.to_string_lossy().into_owned();
        let entry_hash: [u8; 32] = hash(&read(entry)?).into();

        if let Some(node) = self.cache.lookup(&key) {
            let dependencies_are_valid = node.dependencies.iter().all(is_dependency_valid);
            let outputs_are_present = !node.outputs.is_empty()
                && node.outputs.iter().all(|o| Path::new(o).exists());

            if node.hash == entry_hash && dependencies_are_valid && outputs_are_present {
                self.cache.touch(&key, node.hash, node.dependencies);

                info!("Cached GLTF {:?}", task.build_target.relative_full());

                return Ok(());
            }
        }

        let result = self.extract(dispatcher, task, &key, entry_hash);

        if result.is_err() {
            self.cache.invalidate(&key);
        }

        result
    }
}

fn collect_dependencies(entry: &Path, document: &Document) -> Result<Vec<DependencyRecord>> {
    let mut dependencies = Vec::new();

    let bin_path = entry.with_extension("bin");
    if bin_path.exists() {
        dependencies.push(dependency_record(&bin_path)?);
    }

    if let Some(parent) = entry.parent() {
        for image in document.images() {
            if let Source::Uri { uri, .. } = image.source() {
                let image_path = parent.join(uri);

                if image_path.exists() {
                    dependencies.push(dependency_record(&image_path)?);
                }
            }
        }
    }

    dependencies.sort_by(|a, b| a.path.cmp(&b.path));
    dependencies.dedup_by(|a, b| a.path == b.path);

    Ok(dependencies)
}

fn dependency_record(path: &Path) -> Result<DependencyRecord> {
    let path = canonicalize(path)?;
    let bytes = read(&path)?;

    Ok(DependencyRecord {
        path: path.to_string_lossy().into_owned(),
        hash: hash(&bytes).into(),
    })
}
