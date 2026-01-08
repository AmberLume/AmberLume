use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use gltf::{Node, Scene};
use log::{info, warn};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use serde::Deserialize;
use serde_json::from_str;
use crate::build_task::{BuildTask, CollectSceneTask, ExtractModelAssetTask, WriteFileTask};
use crate::data::scene_data::{SceneData, EntityPlaceholderData, PhysicalBodyData, BodyTypeData, BodyColliderData, BodyColliderShapeData};
use crate::dispatcher::Dispatcher;
use crate::paths::Paths;
use crate::processors::processor::Processor;

#[derive(Deserialize, Debug, PartialEq)]
struct PlaceholderExtras {
    pub asset_file_name: String,
    pub collection_name: String,

    pub physical_body: PhysicalBody,
}

#[derive(Deserialize, Debug, PartialEq)]
struct PhysicalBody {
    pub body_type: BodyType,

    pub colliders: Vec<BodyCollider>,
}

#[derive(Deserialize, Debug, PartialEq)]
enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Deserialize, Debug, PartialEq)]
struct BodyCollider {
    pub name: String,

    pub shape: BodyColliderShape,

    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Deserialize, Debug, PartialEq)]
enum BodyColliderShape {
    Box {
        size: [f32; 3],
    },
}

pub struct CollectSceneProcessor;

impl CollectSceneProcessor {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn collect_scene_placeholders(&self, paths: &Paths, scene: &Scene) -> Result<SceneData> {
        let mut placeholders = Vec::new();

        for node in scene.nodes() {
            self.collect_placeholder(&paths, &mut placeholders, &node)?;
        }

        Ok(SceneData {
            name: scene.name().unwrap().to_string(),

            placeholders,
        })
    }

    fn collect_placeholder(&self, paths: &Paths, nodes: &mut Vec<EntityPlaceholderData>, node: &Node) -> Result<()> {
        let name = node.name().expect("Node names are required!").to_string();

        if let Some(extras) = self.extract_placeholder_extras(paths, &node) {
            let asset_key = format!("{}#{}", extras.asset_file_name, extras.collection_name);
            let physical_body = Self::create_physical_body(extras.physical_body);

            let (transform, rotation, scale) = node.transform().decomposed();

            nodes.push(EntityPlaceholderData {
                name,

                transform,
                rotation,
                scale,

                asset_key,

                physical_body,
            });
        }

        for node in node.children() {
            self.collect_placeholder(&paths, nodes, &node)?;
        }

        Ok(())
    }

    fn create_physical_body(physical_body: PhysicalBody) -> PhysicalBodyData {
        let body_type = match physical_body.body_type {
            BodyType::Static => BodyTypeData::Static,
            BodyType::Kinematic => BodyTypeData::Kinematic,
            BodyType::Dynamic => BodyTypeData::Dynamic,
        };

        let colliders = physical_body.colliders.iter().map(|collider| {
            let collider_shape = match collider.shape {
                BodyColliderShape::Box { size } => { BodyColliderShapeData::Box { size } }
            };

            BodyColliderData {
                collider_name: collider.name.clone(),

                collider_shape,

                position: collider.position,
                rotation: collider.rotation,
            }
        }).collect();

        PhysicalBodyData {
            body_type,

            colliders,
        }
    }

    fn extract_placeholder_extras(&self, paths: &Paths, node: &Node) -> Option<PlaceholderExtras> {
        if let Some(raw_extras) = node.extras() {
            let extras = from_str::<PlaceholderExtras>(raw_extras.get());

            if let Ok(extras) = extras {
                Some(extras)
            } else {
                warn!("Error while deserializing placeholder extras to PlaceholderExtras! Path: {}, extras: {:?}", paths.relative.display(), raw_extras);

                None
            }
        } else {
            warn!("Node does not have a placeholder extras! Path: {}, name: {:?}!", paths.relative.display(), node.name());

            None
        }
    }
}

impl Processor<CollectSceneTask> for CollectSceneProcessor {
    fn process(&self, dispatcher: Arc<Dispatcher>, task: &CollectSceneTask) -> Result<()> {
        let document = task.gltf_file.get_document()?;
        let scene = document.scenes().nth(task.scene_index).unwrap();

        info!("Parsing scene {}(index {}) from {}", task.name, task.scene_index, task.paths.relative.display());

        let scene_data = self.collect_scene_placeholders(&task.paths, &scene)?;

        scene_data.placeholders.iter().for_each(|placeholder_data| {
            let (file_name, collection_name) = placeholder_data.asset_key.split_once('#').unwrap();

            let relative_path = PathBuf::from(file_name).join(collection_name).with_extension("gltf");

            let placeholder_paths = Paths::create(
                &relative_path,
                &task.paths.source,
                &task.paths.target.parent().unwrap()
            );

            dispatcher.clone().dispatch(BuildTask::ExtractModelAsset(ExtractModelAssetTask {
                collection_name: collection_name.to_string(),
                file_name: file_name.to_string(),

                paths: placeholder_paths,
            }));
        });

        let scene_bytes = to_bytes::<Error>(&scene_data)?.into_vec();

        let path = task.paths.target
            .join(&task.name)
            .with_extension("scene");

        dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
            target_path: path,

            data: scene_bytes,
        }));

        Ok(())
    }
}
