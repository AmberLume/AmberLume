use std::path::PathBuf;
use std::sync::Arc;
use gltf::Node;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use serde::Deserialize;
use serde_json::from_str;
use tracing::error;
use crate::build_task::{BuildTask, WriteFileTask};
use crate::data::scene_data::{BodyTypeData, EntityPlaceholderData, SceneData};
use crate::processors::assets::link_utils::extract_link_extras;
use anyhow::Result;
use crate::dispatcher::Dispatcher;
use crate::paths::AlpacaPaths;

#[derive(Deserialize, Debug, PartialEq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct MeshPlaceholderExtras {
    pub body_type: BodyType,
}

fn extract_mesh_placeholder_extras(mesh_node: &Node) -> Option<MeshPlaceholderExtras> {
    let extras = mesh_node.extras()
        .as_ref()
        .and_then(|extras| from_str::<MeshPlaceholderExtras>(extras.get()).ok());

    if extras.is_none() {
        error!("Failed to extract MeshPlaceholderExtras. Extras: {:?}", mesh_node.extras());
    }

    extras
}

pub fn write_scene_data(
    dispatcher: Arc<Dispatcher>,
    paths: &AlpacaPaths,
    name: String,
    root: Node,
) -> Result<()> {
    let mut placeholders = Vec::new();

    for placeholder in root.children() {
        let Some(mesh_placeholder_extras) = extract_mesh_placeholder_extras(&placeholder) else { continue };
        let Some(link_extras) = extract_link_extras(&placeholder) else { continue };

        let (transform, rotation, scale) = placeholder.transform().decomposed();
        let (collection_name, _) = link_extras.source_collection.split_once('.').unwrap();

        let link_parent_path = PathBuf::from(link_extras.source_gltf);
        let asset_path = PathBuf::from("assets").join(link_parent_path.parent().unwrap()).join(collection_name);

        let mesh_asset_key = asset_path.with_extension("MESH").to_str().unwrap().to_string();
        let physical_body_asset_key = asset_path.with_extension("PHYSICAL_BODY").to_str().unwrap().to_string();

        let physical_body_type = match mesh_placeholder_extras.body_type {
            BodyType::Static => BodyTypeData::Static,
            BodyType::Kinematic => BodyTypeData::Kinematic,
            BodyType::Dynamic => BodyTypeData::Dynamic,
        };

        let entity_placeholder_data = EntityPlaceholderData {
            name: link_extras.source_collection,

            transform,
            rotation,
            scale,

            mesh_asset_key,

            physical_body_type,
            physical_body_asset_key,
        };

        placeholders.push(entity_placeholder_data);
    }

    let path = paths.target.join(&name).with_extension("SCENE");
    let scene_data = SceneData {
        name,

        placeholders,
    };

    let mesh_bytes = to_bytes::<Error>(&scene_data)?.into_vec();

    dispatcher.dispatch(BuildTask::WriteFile(WriteFileTask {
        target_path: path,

        data: mesh_bytes,
    }));

    Ok(())
}
