use std::collections::HashMap;
use anyhow::{bail, Result};
use glam::{Mat4, Quat, Vec3};
use gltf::{Document, Node};
use resource_data::skeleton_data::BoneData;

const ROOT_BONE: &str = "root";

#[derive(Debug, PartialEq)]
pub struct Skeleton {
    pub name: String,
    pub bones: Vec<BoneData>,
}

impl Skeleton {
    pub fn adapt(node: &Node, document: &Document, bin: Option<&[u8]>) -> Result<Skeleton> {
        let Some(name) = node.name() else {
            bail!("Skeleton must have a name");
        };

        let Some(root) = node.children().find(|child| child.name() == Some(ROOT_BONE)) else {
            bail!("Skeleton {} must have a bone named {}", name, ROOT_BONE);
        };

        let mut bone_nodes = Vec::new();
        collect_bone_nodes(&root, &mut bone_nodes);

        let parent_indices = collect_parent_indices(&bone_nodes);
        let inverse_bind_matrices = collect_inverse_bind_matrices(document, bin);

        let mut bones = Vec::with_capacity(bone_nodes.len());

        for index in 0..bone_nodes.len() {
            let bone = &bone_nodes[index];

            let Some(bone_name) = bone.name() else {
                bail!("All bones must have names! Skeleton: {}", name);
            };

            let inverse_bind_matrix = match inverse_bind_matrices.get(&bone.index()) {
                Some(matrix) => matrix.to_cols_array_2d(),
                None => world_matrix(&bone_nodes, &parent_indices, index).inverse().to_cols_array_2d(),
            };

            bones.push(BoneData {
                name: bone_name.to_string(),
                parent_index: parent_indices[index],
                inverse_bind_matrix,
            });
        }

        Ok(Self {
            name: name.to_string(),
            bones,
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

fn collect_inverse_bind_matrices(document: &Document, bin: Option<&[u8]>) -> HashMap<usize, Mat4> {
    let Some(skin) = document.skins().next() else {
        return HashMap::new();
    };

    let reader = skin.reader(|buffer| match buffer.source() {
        gltf::buffer::Source::Bin => None,
        gltf::buffer::Source::Uri(_) => bin,
    });

    let matrices: Vec<Mat4> = reader
        .read_inverse_bind_matrices()
        .map(|iter| iter.map(|matrix| Mat4::from_cols_array_2d(&matrix)).collect())
        .unwrap_or_default();

    skin.joints()
        .enumerate()
        .filter_map(|(index, joint)| matrices.get(index).map(|matrix| (joint.index(), *matrix)))
        .collect()
}

fn world_matrix(bone_nodes: &[Node], parent_indices: &[i32], index: usize) -> Mat4 {
    let mut chain = vec![index];
    let mut current = index;

    while parent_indices[current] >= 0 {
        current = parent_indices[current] as usize;
        chain.push(current);
    }

    let mut world = Mat4::IDENTITY;

    for &bone in chain.iter().rev() {
        let (translation, rotation, scale) = bone_nodes[bone].transform().decomposed();

        world = world * Mat4::from_scale_rotation_translation(
            Vec3::from(scale),
            Quat::from_array(rotation),
            Vec3::from(translation),
        );
    }

    world
}
