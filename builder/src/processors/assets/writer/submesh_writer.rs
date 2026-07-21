use crate::dispatcher::Dispatcher;
use std::sync::Arc;
use crate::build_target::BuildTarget;
use resource_data::submesh_data::SubmeshData;
use crate::processors::assets::adapter::submesh_adapter::Submesh;
use crate::processors::assets::writer::material_writer::write_material_data;

pub fn collect_submesh_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    submesh: Submesh,
) -> SubmeshData {
    let material = write_material_data(dispatcher, build_target, &submesh.material);

    SubmeshData {
        material,

        indices: submesh.indices,
        positions: submesh.positions,
        normals: submesh.normals,
        tangents: submesh.tangents,
        uvs: submesh.uvs,
        bone_indices: submesh.bone_indices,
        bone_weights: submesh.bone_weights,

        bounds: submesh.bounds,
    }
}
