use anyhow::{bail, Result};
use glam::{Mat4, Vec3};
use resource_data::submesh_data::SubmeshData;

pub fn calculate_aabb<I>(vertices: I) -> [f32; 6]
where
    I: IntoIterator<Item = [f32; 3]>
{
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for [x, y, z] in vertices {
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        min[2] = min[2].min(z);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
        max[2] = max[2].max(z);
    }

    [min[0], min[1], min[2], max[0], max[1], max[2]]
}

pub fn calculate_global_aabb<I>(aabbs: I) -> [f32; 6]
where
    I: IntoIterator<Item = [f32; 6]>
{
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    for bounds in aabbs {
        min[0] = min[0].min(bounds[0]);
        min[1] = min[1].min(bounds[1]);
        min[2] = min[2].min(bounds[2]);
        max[0] = max[0].max(bounds[3]);
        max[1] = max[1].max(bounds[4]);
        max[2] = max[2].max(bounds[5]);
    }

    [min[0], min[1], min[2], max[0], max[1], max[2]]
}

pub fn calculate_bone_aabbs(
    submeshes: &[SubmeshData],
    inverse_bind_matrices: &[[[f32; 4]; 4]],
) -> Result<Vec<[f32; 6]>> {
    let mut bone_positions = vec![Vec::new(); inverse_bind_matrices.len()];

    for submesh in submeshes {
        let vertices = submesh.positions.iter()
            .zip(&submesh.bone_indices)
            .zip(&submesh.bone_weights);

        for ((position, bone_indices), bone_weights) in vertices {
            for (&bone_index, &bone_weight) in bone_indices.iter().zip(bone_weights) {
                if bone_weight <= 0.0 {
                    continue;
                }

                let bone_index = bone_index as usize;

                let Some(inverse_bind_matrix) = inverse_bind_matrices.get(bone_index) else {
                    bail!("Vertex is bound to bone {} of {}", bone_index, inverse_bind_matrices.len());
                };

                let bone_position = Mat4::from_cols_array_2d(inverse_bind_matrix)
                    .transform_point3(Vec3::from(*position));

                bone_positions[bone_index].push(bone_position.to_array());
            }
        }
    }

    Ok(bone_positions.into_iter().map(calculate_aabb).collect())
}
