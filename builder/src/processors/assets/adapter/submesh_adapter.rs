use anyhow::{bail, Result};
use glam::Vec3;
use gltf::{Primitive, buffer};
use crate::processors::assets::adapter::material_adapter::Material;
use crate::processors::assets::utils::aabb_utils::calculate_aabb;

#[derive(Debug, PartialEq)]
pub struct Submesh {
    pub indices: Vec<u32>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tangents: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
    pub bone_indices: Vec<[u16; 4]>,
    pub bone_weights: Vec<[f32; 4]>,

    pub bounds: [f32; 6],

    pub material: Material,
}

impl Submesh {
    pub fn adapt(
        primitive: &Primitive,
        bin: Option<&[u8]>,
        sorted_bone_names: &[String],
        bone_names: &[String],
    ) -> Result<Submesh> {
        let material = Material::adapt(&primitive.material());

        let reader = primitive.reader(|buffer| match buffer.source() {
            buffer::Source::Bin => None,
            buffer::Source::Uri(_) => bin,
        });

        let Some(indices) = reader.read_indices() else {
            bail!("Accessor for indices coordinates not found");
        };

        let Some(positions) = reader.read_positions() else {
            bail!("Accessor for positions not found");
        };

        let Some(normals) = reader.read_normals() else {
            bail!("Accessor for normal not found");
        };

        let indices = indices.into_u32().collect::<Vec<u32>>();
        let positions = positions.collect::<Vec<[f32; 3]>>();
        let normals = normals.collect::<Vec<[f32; 3]>>();

        let uvs = match reader.read_tex_coords(0) {
            Some(uvs) => uvs.into_f32().collect::<Vec<[f32; 2]>>(),
            None if material.base.is_some() || material.normal.is_some() || material.orm.is_some() => {
                bail!("Material {} uses textures but the primitive has no texture coordinates", material.name)
            }
            None => vec![[0.0, 0.0]; positions.len()],
        };

        let tangents = match reader.read_tangents() {
            Some(tangents) => tangents.collect::<Vec<[f32; 4]>>(),
            None if material.normal.is_some() => {
                bail!("Material {} has a normal texture but the primitive has no tangents", material.name)
            }
            None => normals.iter().map(|normal| {
                let tangent = Vec3::from(*normal)
                    .try_normalize()
                    .unwrap_or(Vec3::Z)
                    .any_orthonormal_vector();

                [tangent.x, tangent.y, tangent.z, 1.0]
            }).collect(),
        };

        let bone_indices = match reader.read_joints(0) {
            Some(joints) => remap_bones(joints.into_u16().collect(), sorted_bone_names, bone_names),
            None => vec![[0u16; 4]; positions.len()],
        };

        let bone_weights = match reader.read_weights(0) {
            Some(weights) => weights.into_f32().collect(),
            None => vec![[1.0, 0.0, 0.0, 0.0]; positions.len()],
        };

        if positions.len() != uvs.len() || uvs.len() != normals.len() || normals.len() != tangents.len() {
            bail!(
                "Submesh arrays are not equal! Positions: {}, normals: {}, tangents: {}, UVs: {}",
                positions.len(),
                normals.len(),
                tangents.len(),
                uvs.len(),
            );
        }

        let bounds = calculate_aabb(positions.iter().copied());

        Ok(Self {
            indices,
            positions,
            normals,
            tangents,
            uvs,
            bone_indices,
            bone_weights,

            bounds,

            material,
        })
    }
}

fn remap_bones(
    raw: Vec<[u16; 4]>,
    sorted_bone_names: &[String],
    bone_names: &[String],
) -> Vec<[u16; 4]> {
    if bone_names.is_empty() || sorted_bone_names.is_empty() {
        return raw;
    }

    let remap: Vec<u16> = bone_names
        .iter()
        .map(|name| sorted_bone_names.iter().position(|bone| bone == name).unwrap_or(0) as u16)
        .collect();

    raw.into_iter()
        .map(|joint_index| [
            remap.get(joint_index[0] as usize).copied().unwrap_or(0),
            remap.get(joint_index[1] as usize).copied().unwrap_or(0),
            remap.get(joint_index[2] as usize).copied().unwrap_or(0),
            remap.get(joint_index[3] as usize).copied().unwrap_or(0),
        ])
        .collect()
}
