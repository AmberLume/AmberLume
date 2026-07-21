use std::sync::Arc;
use gltf::Node;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use serde::Deserialize;
use serde_json::{from_slice, from_str};
use tracing::error;
use crate::build_task::BuildTask;
use resource_data::scene_data::{BodyTypeData, EntityPlaceholderData, SceneData};
use anyhow::Result;
use crate::build_target::BuildTarget;
use crate::dispatcher::Dispatcher;
use std::collections::HashMap;
use std::fs::read;
use std::path::{Path, PathBuf};
use crate::processors::utils::{resource_key, resource_key_from_extras};
use crate::processors::assets::link_utils::LinkExtras;

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
    build_target: &BuildTarget,
    name: String,
    root: Node,
) -> Result<()> {
    let mut placeholders = Vec::new();

    for placeholder in root.children() {
        let Some(mesh_placeholder_extras) = extract_mesh_placeholder_extras(&placeholder) else { continue };
        let Some(link_extras) = LinkExtras::from_node(&placeholder) else { continue };

        let (transform, rotation, scale) = placeholder.transform().decomposed();

        let mesh = resource_key_from_extras(&build_target, &link_extras, "MESH");
        let physical_body = resource_key_from_extras(&build_target, &link_extras, "PHYSICAL_BODY");

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

            mesh: Some(mesh),

            physical_body_type,
            physical_body,
        };

        placeholders.push(entity_placeholder_data);
    }

    let resource_key = resource_key(build_target, &name, "SCENE");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&SceneData {
            name: name.clone(),

            placeholders,
        })?.to_vec(),
    ));

    Ok(())
}

pub fn write_scene_data_flat(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    placeholder_nodes: Vec<Node>,
) -> Result<()> {
    let name = build_target.name.clone();

    let mut mesh_presence: HashMap<PathBuf, bool> = HashMap::new();
    let mut placeholders = Vec::new();

    for placeholder in &placeholder_nodes {
        let Some(mesh_placeholder_extras) = extract_mesh_placeholder_extras(placeholder) else { continue };
        let Some(link_extras) = LinkExtras::from_node(placeholder) else { continue };

        let relative = PathBuf::from(&link_extras.source_gltf);

        let Some(referenced) = build_target.to_relative(&relative) else {
            error!("Placeholder references missing asset file {:?}", link_extras.source_gltf);

            continue;
        };

        let reference_name = referenced.name.clone();

        let has_mesh = *mesh_presence
            .entry(referenced.entry.clone())
            .or_insert_with(|| gltf_has_mesh_node(&referenced.entry));

        let mesh = if has_mesh {
            Some(resource_key(&referenced, &reference_name, "MESH"))
        } else {
            None
        };

        let physical_body = resource_key(&referenced, &reference_name, "PHYSICAL_BODY");

        let (transform, rotation, scale) = placeholder.transform().decomposed();

        let physical_body_type = match mesh_placeholder_extras.body_type {
            BodyType::Static => BodyTypeData::Static,
            BodyType::Kinematic => BodyTypeData::Kinematic,
            BodyType::Dynamic => BodyTypeData::Dynamic,
        };

        placeholders.push(EntityPlaceholderData {
            name: reference_name,

            transform,
            rotation,
            scale,

            mesh,

            physical_body_type,
            physical_body,
        });
    }

    let resource_key = resource_key(build_target, &name, "SCENE");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&SceneData {
            name: name.clone(),

            placeholders,
        })?.to_vec(),
    ));

    Ok(())
}

fn gltf_has_mesh_node(path: &Path) -> bool {
    let Ok(bytes) = read(path) else { return false };
    let Ok(value) = from_slice::<serde_json::Value>(&bytes) else { return false };
    let Some(nodes) = value.get("nodes").and_then(|nodes| nodes.as_array()) else { return false };

    nodes.iter().any(|node| {
        node.get("extras")
            .and_then(|extras| extras.get("amberlume_type"))
            .and_then(|value| value.as_str())
            == Some("Mesh")
    })
}
