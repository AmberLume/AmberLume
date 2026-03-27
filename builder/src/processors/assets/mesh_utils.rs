use crate::processors::assets::aabb_utils::calculate_global_aabb;
use crate::data::mesh_data::MeshData;
use crate::dispatcher::Dispatcher;
use crate::processors::assets::submesh_utils::collect_submesh_data;
use gltf::Node;
use std::sync::Arc;
use tracing::error;
use anyhow::{bail, Result};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use crate::processors::utils::resource_key;

pub fn write_mesh_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    name: String,
    root: &Node,
    bin: Option<&[u8]>,
) -> Result<()> {
    let Some(meshes_root) = get_meshes_root(&root) else {
        bail!("Failed to find meshes root path for {}", name);
    };

    let mut submeshes = Vec::new();

    for mesh_node in meshes_root.children() {
        if let Some(mesh) = mesh_node.mesh() {
            submeshes.extend(mesh.primitives().map(|primitive| {
                collect_submesh_data(dispatcher.clone(), &build_target, bin, &primitive).unwrap()
            }))
        }
    }

    let bounds = calculate_global_aabb(submeshes.iter().map(|m| m.bounds));

    let resource_key = resource_key(build_target, &name, "MESH");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&MeshData {
            name: name.clone(),

            submeshes,

            bounds,
        })?.to_vec(),
    ));

    Ok(())
}

fn get_meshes_root<'a>(mesh_root: &Node<'a>) -> Option<Node<'a>> {
    mesh_root
        .children()
        .find(|node| {
            node.name()
                .map(|name| name.ends_with(".Meshes"))
                .unwrap_or(false)
        })
        .or_else(|| {
            error!("Failed to parse meshless node. Name: {:?}", mesh_root.name());

            None
        })
}
