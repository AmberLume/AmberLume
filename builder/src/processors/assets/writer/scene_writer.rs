use std::sync::Arc;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use serde_json::from_slice;
use tracing::error;
use crate::build_task::BuildTask;
use resource_data::scene_data::{EntityPlaceholderData, SceneData};
use anyhow::Result;
use crate::build_target::BuildTarget;
use crate::dispatcher::Dispatcher;
use std::collections::HashMap;
use std::fs::read;
use std::path::{Path, PathBuf};
use crate::processors::utils::resource_key;
use crate::processors::assets::adapter::placeholder_adapter::Placeholder;

pub fn write_scene_data_flat(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    descriptions: Vec<Placeholder>,
) -> Result<()> {
    let name = build_target.name.clone();

    let mut mesh_presence: HashMap<PathBuf, bool> = HashMap::new();
    let mut placeholders = Vec::new();

    for description in descriptions {
        let relative = PathBuf::from(&description.source_gltf);

        let Some(referenced) = build_target.to_relative(&relative) else {
            error!("Placeholder references missing asset file {:?}", description.source_gltf);

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

        placeholders.push(EntityPlaceholderData {
            name: reference_name,

            transform: description.translation,
            rotation: description.rotation,
            scale: description.scale,

            mesh,

            physical_body_type: description.body_type,
            physical_body,
        });
    }

    dispatch_scene(dispatcher, build_target, name, placeholders)
}

fn dispatch_scene(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    name: String,
    placeholders: Vec<EntityPlaceholderData>,
) -> Result<()> {
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
