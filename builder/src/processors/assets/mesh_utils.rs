use crate::aabb_utils::calculate_global_aabb;
use crate::data::mesh_data::MeshData;
use crate::dispatcher::Dispatcher;
use crate::paths::Paths;
use crate::processors::assets::submesh_utils::create_submesh_data;
use gltf::Node;
use std::sync::Arc;
use tracing::error;
use anyhow::{bail, Result};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_task::{BuildTask, WriteFileTask};

pub fn write_mesh_data(
    dispatcher: Arc<Dispatcher>,
    paths: &Paths,
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
                create_submesh_data(dispatcher.clone(), &paths, bin, &primitive).unwrap()
            }))
        }
    }

    let bounds = calculate_global_aabb(submeshes.iter().map(|m| m.bounds));

    let path = paths.target.join(&name).with_extension("MESH");
    let mesh_data = MeshData {
        name,

        submeshes,

        bounds,
    };

    let mesh_bytes = to_bytes::<Error>(&mesh_data)?.into_vec();

    dispatcher.clone().dispatch(BuildTask::WriteFile(WriteFileTask {
        target_path: path,

        data: mesh_bytes,
    }));

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
