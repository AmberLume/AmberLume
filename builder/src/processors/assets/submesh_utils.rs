use crate::processors::assets::aabb_utils::calculate_aabb;
use crate::dispatcher::Dispatcher;
use anyhow::Result;
use anyhow::bail;
use gltf::{Primitive, buffer};
use std::sync::Arc;
use crate::build_target::BuildTarget;
use resource_data::submesh_data::SubmeshData;
use crate::processors::assets::material_utils::write_material_data;

pub fn collect_submesh_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    bin: Option<&[u8]>,
    primitive: &Primitive,
    sorted_bone_names: &Vec<String>,
    bone_names: &Vec<String>,
) -> Result<SubmeshData> {
    let reader = primitive.reader(|buffer| match buffer.source() {
        buffer::Source::Bin => None,
        buffer::Source::Uri(_) => bin,
    });

    let indices = if let Some(indices) = reader.read_indices() {
        indices.into_u32().collect::<Vec<u32>>()
    } else {
        bail!("Accessor for indices coordinates not found");
    };

    let positions: Vec<[f32; 3]> = if let Some(positions) = reader.read_positions() {
        positions.collect::<Vec<[f32; 3]>>()
    } else {
        bail!("Accessor for positions not found");
    };

    let uvs: Vec<[f32; 2]> = if let Some(texture_coordinates) = reader.read_tex_coords(0) {
        texture_coordinates.into_f32().collect()
    } else {
        bail!("Accessor for texture coordinates not found");
    };

    let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
        iter.collect::<Vec<[f32; 3]>>()
    } else {
        bail!("Accessor for normal not found");
    };

    let tangents: Vec<[f32; 4]> = if let Some(iter) = reader.read_tangents() {
        iter.collect::<Vec<[f32; 4]>>()
    } else {
        bail!("Accessor for tangent not found");
    };

    let bone_indices: Vec<[u16; 4]> = if let Some(iter) = reader.read_joints(0) {
        let raw: Vec<[u16; 4]> = iter.into_u16().collect();

        if !bone_names.is_empty() && !sorted_bone_names.is_empty() {
            let remap: Vec<u16> = bone_names.iter()
                .map(|name| sorted_bone_names.iter()
                    .position(|bone| bone == name)
                    .unwrap_or(0) as u16)
                .collect();

            raw.into_iter().map(|joint_index| [
                remap.get(joint_index[0] as usize).copied().unwrap_or(0),
                remap.get(joint_index[1] as usize).copied().unwrap_or(0),
                remap.get(joint_index[2] as usize).copied().unwrap_or(0),
                remap.get(joint_index[3] as usize).copied().unwrap_or(0),
            ]).collect()
        } else {
            raw
        }
    } else {
        vec![[0u16; 4]; positions.len()]
    };

    let bone_weights: Vec<[f32; 4]> = if let Some(iter) = reader.read_weights(0) {
        iter.into_f32().collect()
    } else {
        vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]
    };

    let positions_count = positions.len();
    let uvs_count = uvs.len();
    let normals_count = normals.len();
    let tangents_count = tangents.len();

    assert!(
        positions_count == uvs_count
            && uvs_count == normals_count
            && normals_count == tangents_count,
        "Submesh arrays are not equals! Positions: {}, normals: {}, tangents: {}, UVs: {}",
        positions_count,
        normals_count,
        tangents_count,
        uvs_count,
    );

    let bounds = calculate_aabb(positions.iter().copied());

    let material = write_material_data(dispatcher, &build_target, primitive.material());

    Ok(SubmeshData {
        material,

        indices,
        positions,
        normals,
        tangents,
        uvs,
        bone_indices,
        bone_weights,

        bounds,
    })
}
