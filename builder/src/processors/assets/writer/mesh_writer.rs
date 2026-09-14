use crate::processors::assets::utils::aabb_utils::calculate_global_aabb;
use resource_data::mesh_data::MeshData;
use resource_data::resource_key::ResourceKey;
use crate::dispatcher::Dispatcher;
use crate::processors::assets::writer::submesh_writer::collect_submesh_data;
use std::sync::Arc;
use anyhow::{bail, Result};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use crate::processors::utils::resource_key;
use crate::processors::assets::adapter::mesh_adapter::Mesh;

pub fn write_mesh_data_flat(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    meshes: Vec<Mesh>,
    skeleton: Option<ResourceKey>,
) -> Result<()> {
    let name = build_target.name.clone();

    let mut submeshes = Vec::new();
    let mut inverse_bind_matrices = Vec::new();

    for mesh in meshes {
        if !mesh.inverse_bind_matrices.is_empty() {
            if !inverse_bind_matrices.is_empty() && inverse_bind_matrices != mesh.inverse_bind_matrices {
                bail!("Skinned meshes of {} have different bind poses", name);
            }

            inverse_bind_matrices = mesh.inverse_bind_matrices;
        }

        for submesh in mesh.submeshes {
            submeshes.push(collect_submesh_data(dispatcher.clone(), build_target, submesh));
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

            skeleton,
            inverse_bind_matrices,

            bounds,
        })?.to_vec(),
    ));

    Ok(())
}

