use crate::processors::assets::aabb_utils::calculate_global_aabb;
use crate::data::mesh_data::MeshData;
use crate::dispatcher::Dispatcher;
use crate::processors::assets::submesh_utils::collect_submesh_data;
use gltf::{Node, Skin};
use std::sync::Arc;
use tracing::error;
use anyhow::{bail, Result};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use crate::processors::assets::bones_utils::collect_bone_nodes;
use crate::processors::utils::{resource_key, resource_key_from_extras};
use crate::processors::assets::link_utils::LinkExtras;

pub fn write_mesh_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    name: String,
    root: &Node,
    skeletons: Vec<Skin>,
    bin: Option<&[u8]>,
) -> Result<()> {
    let Some(meshes_root) = get_meshes_root(&root) else {
        bail!("Failed to find meshes root path for {}", name);
    };

    if skeletons.len() > 1 {
        bail!("More than 1 skeletons found for {}", name);
    }

    let sorted_bone_names = skeletons.first()
        .and_then(|skin| skin.joints().find(|j| j.name() == Some("root")))
        .map(|root| {
            let mut nodes = Vec::new();

            collect_bone_nodes(&root, &mut nodes);

            nodes.iter().map(|node| node.name().unwrap_or("").to_string()).collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let bone_names = skeletons.first()
        .map(|skin| skin.joints().map(|j| j.name().unwrap_or("").to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let mut submeshes = Vec::new();

    for mesh_node in meshes_root.children() {
        if let Some(mesh) = mesh_node.mesh() {
            submeshes.extend(mesh.primitives().map(|primitive| {
                collect_submesh_data(dispatcher.clone(), &build_target, bin, &primitive, &sorted_bone_names, &bone_names).unwrap()
            }))
        }
    }

    let bounds = calculate_global_aabb(submeshes.iter().map(|m| m.bounds));

    let skeleton_resource_key = node_with_name(root, |name| name.ends_with(".SKELETON")).and_then(|node| {
        let Some(link_extras) = LinkExtras::from_node(&node) else {
            error!("Skeleton must have link extras");

            return None
        };

        Some(resource_key_from_extras(&build_target, &link_extras, "SKELETON"))
    });

    let resource_key = resource_key(build_target, &name, "MESH");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&MeshData {
            name: name.clone(),

            submeshes,

            skeleton: skeleton_resource_key,

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

fn node_with_name<'a>(parent: &Node<'a>, condition: impl Fn(&str) -> bool) -> Option<Node<'a>> {
    parent.children().find(|child| child.name().map(|n| condition(n)).unwrap_or(false))
}
