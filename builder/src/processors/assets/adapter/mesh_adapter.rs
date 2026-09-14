use anyhow::{bail, Context, Result};
use gltf::Node;
use crate::processors::assets::adapter::skeleton_adapter::{collect_bone_nodes, ROOT_BONE};
use crate::processors::assets::adapter::submesh_adapter::Submesh;

#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub submeshes: Vec<Submesh>,
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

impl Mesh {
    pub fn adapt(node: &Node, bin: Option<&[u8]>) -> Result<Mesh> {
        let mut sorted_bone_names = Vec::new();
        let mut bone_names = Vec::new();
        let mut inverse_bind_matrices = Vec::new();

        if let Some(skin) = node.skin() {
            let Some(root) = skin.joints().find(|joint| joint.name() == Some(ROOT_BONE)) else {
                bail!("Skin must contain a bone named {}", ROOT_BONE);
            };

            let mut skeleton_bones = Vec::new();
            collect_bone_nodes(&root, &mut skeleton_bones);

            sorted_bone_names = skeleton_bones.iter()
                .map(|bone| bone.name().unwrap_or_default().to_string())
                .collect();

            bone_names = skin.joints()
                .map(|joint| joint.name().unwrap_or_default().to_string())
                .collect();

            let reader = skin.reader(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => None,
                gltf::buffer::Source::Uri(_) => bin,
            });

            let joint_inverse_bind_matrices = reader
                .read_inverse_bind_matrices()
                .context("Skin must have inverse bind matrices")?
                .collect::<Vec<_>>();

            for bone_name in &sorted_bone_names {
                let Some(joint_index) = bone_names.iter().position(|name| name == bone_name) else {
                    bail!("Skin has no joint for bone {}", bone_name);
                };

                let Some(inverse_bind_matrix) = joint_inverse_bind_matrices.get(joint_index) else {
                    bail!("Skin has no inverse bind matrix for bone {}", bone_name);
                };

                inverse_bind_matrices.push(*inverse_bind_matrix);
            }
        }

        let mut submeshes = Vec::new();

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                submeshes.push(Submesh::adapt(&primitive, bin, &sorted_bone_names, &bone_names)?);
            }
        }

        Ok(Self {
            submeshes,
            inverse_bind_matrices,
        })
    }
}
