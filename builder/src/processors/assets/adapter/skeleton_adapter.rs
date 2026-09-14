use anyhow::{bail, Result};
use gltf::Node;
use resource_data::skeleton_data::BoneData;
use crate::processors::assets::extras_adapter::skeleton_extras_adapter::SkeletonExtras;

pub(super) const ROOT_BONE: &str = "root";

#[derive(Debug, PartialEq)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<BoneData>,

    pub source_gltf: Option<String>,
}

impl Skeleton {
    pub fn adapt(node: &Node) -> Result<Skeleton> {
        let Some(name) = node.name() else {
            bail!("Skeleton must have a name");
        };

        let skeleton_extras = SkeletonExtras::adapt(node)?;

        let Some(root) = node.children().find(|child| child.name() == Some(ROOT_BONE)) else {
            bail!("Skeleton {} must have a bone named {}", name, ROOT_BONE);
        };

        let mut bone_nodes = Vec::new();
        collect_bone_nodes(&root, &mut bone_nodes);

        let parent_indices = collect_parent_indices(&bone_nodes);

        let mut bones = Vec::with_capacity(bone_nodes.len());

        for index in 0..bone_nodes.len() {
            let bone = &bone_nodes[index];

            let Some(bone_name) = bone.name() else {
                bail!("All bones must have names! Skeleton: {}", name);
            };

            let (rest_translation, rest_rotation, rest_scale) = bone.transform().decomposed();

            bones.push(BoneData {
                name: bone_name.to_string(),
                parent_index: parent_indices[index],

                rest_translation,
                rest_rotation,
                rest_scale,
            });
        }

        Ok(Self {
            name: name.to_string(),
            bones,

            source_gltf: skeleton_extras.source_gltf,
        })
    }
}

pub fn collect_bone_nodes<'a>(node: &Node<'a>, result: &mut Vec<Node<'a>>) {
    result.push(node.clone());

    let mut children = node.children().collect::<Vec<_>>();
    children.sort_by_key(|child| child.name().unwrap_or(""));

    for child in children {
        collect_bone_nodes(&child, result);
    }
}

fn collect_parent_indices(bone_nodes: &[Node]) -> Vec<i32> {
    bone_nodes
        .iter()
        .map(|bone| {
            bone_nodes
                .iter()
                .position(|parent| parent.children().any(|child| child.index() == bone.index()))
                .map(|index| index as i32)
                .unwrap_or(-1)
        })
        .collect()
}
