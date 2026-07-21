use std::fs::{canonicalize, read};
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use blake3::hash;
use gltf::{Document, Node};
use gltf::image::Source;
use serde::Deserialize;
use serde_json::from_str;
use tracing::{error, info, warn};
use crate::build_task::ExtractAssetsTask;
use crate::cache::{is_dependency_valid, Cache, DependencyRecord};
use crate::dispatcher::Dispatcher;
use crate::processors::assets::animations_utils::write_animation_data;
use crate::processors::assets::gltf_file::GltfFile;
use crate::processors::assets::bones_utils::write_bones_data;
use crate::processors::assets::mesh_utils::{write_mesh_data, write_mesh_data_flat};
use crate::processors::assets::physical_body_utils::{write_physical_body_data, write_physical_body_data_flat};
use crate::processors::assets::scene_utils::{write_scene_data, write_scene_data_flat};
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

    fn clean_name(name: &str) -> String {
        let parts: Vec<&str> = name.rsplitn(2, '.').collect();

        if parts.len() == 2 && parts[0].chars().all(|c| c.is_ascii_digit()) {
            parts[1].to_string()
        } else {
            name.to_string()
        }
    }

    fn name_and_suffix(name: &str) -> Option<(&str, &str)> {
        let parts = name.rsplit_once('.');

        if let Some((name, suffix)) = parts {
            Some((name, suffix))
        } else {
            None
        }
    }
}

impl Processor<ExtractAssetsTask> for ExtractAssetsProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &ExtractAssetsTask) -> Result<()> {
        let entry = &task.build_target.entry;
        let key = entry.to_string_lossy().into_owned();
        let entry_hash: [u8; 32] = hash(&read(entry)?).into();

        if let Some(node) = self.cache.lookup(&key) {
            let dependencies_are_valid = node.dependencies.iter().all(is_dependency_valid);
            let outputs_are_present = node.outputs.iter().all(|o| Path::new(o).exists());

            if node.hash == entry_hash && dependencies_are_valid && outputs_are_present {
                self.cache.touch(&key, node.hash, node.dependencies);

                info!("Cached GLTF {:?}", task.build_target.relative_full());

                return Ok(());
            }
        }

        info!("Parsing GLTF {:?}", task.build_target.relative_full());

        let gltf_file = Arc::new(GltfFile::create(&task.build_target.entry)?);

        let document = gltf_file.get_document()?;

        let dependencies = collect_dependencies(entry, &document)?;

        self.cache.touch(&key, entry_hash, dependencies);
        self.cache.clear_outputs(&key);

        let mut mesh_nodes = Vec::new();
        let mut collider_nodes = Vec::new();
        let mut placeholder_nodes = Vec::new();

        for node in document.nodes() {
            match node_type(&node).as_deref() {
                Some("Mesh") => mesh_nodes.push(node),
                Some("Collider") => collider_nodes.push(node),
                Some("Placeholder") => placeholder_nodes.push(node),
                _ => {}
            }
        }

        if !mesh_nodes.is_empty() || !collider_nodes.is_empty() || !placeholder_nodes.is_empty() {
            if !placeholder_nodes.is_empty() {
                info!("Importing SCENE (flag) {:?}", task.build_target.relative_full());

                write_scene_data_flat(dispatcher.clone(), &task.build_target, placeholder_nodes)?;
            } else {
                if !mesh_nodes.is_empty() {
                    info!("Importing MESH (flag) {:?}", task.build_target.relative_full());

                    write_mesh_data_flat(dispatcher.clone(), &task.build_target, mesh_nodes, gltf_file.bin())?;
                }

                write_physical_body_data_flat(dispatcher.clone(), &task.build_target, collider_nodes, gltf_file.bin())?;
            }

            return Ok(());
        }

        for scene in document.scenes() {
            let scene_node = scene.nodes().next();

            let collections = if let Some(scene_node) = scene_node {
                scene_node.children()
            } else {
                error!("No scene node found in glTF file. File: {:?}", task.build_target.relative_full());

                continue
            };

            for collection in collections {
                let dispatcher = dispatcher.clone();

                let full_name = if let Some(name) = collection.name() {
                    Self::clean_name(name)
                } else {
                    error!("Found asset node without name! File: {:?}, scene: {:?}", task.build_target.relative_full(), scene.name());

                    continue;
                };

                let (name, suffix) = if let Some((name, suffix)) = Self::name_and_suffix(&full_name) {
                    (name, suffix)
                } else {
                    error!("Cannot extract suffix from name {}.", full_name);

                    continue;
                };

                match suffix {
                    "SCENE" => {
                        info!("Importing SCENE {:?}", name);

                        write_scene_data(dispatcher.clone(), &task.build_target, name.to_string(), collection)?;
                    },
                    "MESH" => {
                        info!("Importing MESH {:?}", name);

                        let skeletons = document.skins().collect::<Vec<_>>();

                        write_mesh_data(dispatcher.clone(), &task.build_target, name.to_string(), &collection, skeletons, gltf_file.bin())?;

                        write_physical_body_data(dispatcher.clone(), &task.build_target, name.to_string(), &collection, gltf_file.bin())?;
                    },
                    "SKELETON" => {
                        info!("Importing SKELETON {:?}", name);

                        let skeletons = document.skins().collect::<Vec<_>>();

                        let skeleton_data = write_bones_data(dispatcher.clone(), &task.build_target, name.to_string(), &collection, skeletons, gltf_file.bin())?;

                        for animation in document.animations() {
                            write_animation_data(dispatcher.clone(), &task.build_target, &skeleton_data, &animation, gltf_file.bin())?;
                        }
                    },
                    _ => {
                        warn!("Unexpected suffix {:?}", suffix);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
struct TypeTag {
    amberlume_type: String,
}

fn node_type(node: &Node) -> Option<String> {
    node.extras()
        .as_ref()
        .and_then(|extras| from_str::<TypeTag>(extras.get()).ok())
        .map(|tag| tag.amberlume_type)
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
