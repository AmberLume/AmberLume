use anyhow::{bail, Result};
use gltf::Node;
use crate::processors::assets::adapter::skeleton_adapter::{collect_bone_nodes, ROOT_BONE};
use crate::processors::assets::adapter::submesh_adapter::Submesh;

#[derive(Debug, PartialEq)]
pub struct Mesh {
    pub submeshes: Vec<Submesh>,
}

impl Mesh {
    pub fn adapt(node: &Node, bin: Option<&[u8]>) -> Result<Mesh> {
        let mut sorted_bone_names = Vec::new();
        let mut bone_names = Vec::new();

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
        }

        let mut submeshes = Vec::new();

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                submeshes.push(Submesh::adapt(&primitive, bin, &sorted_bone_names, &bone_names)?);
            }
        }

        Ok(Self {
            submeshes,
        })
    }
}
