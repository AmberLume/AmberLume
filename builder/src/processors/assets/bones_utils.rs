use crate::dispatcher::Dispatcher;
use crate::paths::AlpacaPaths;
use anyhow::Result;
use anyhow::bail;
use gltf::Node;
use std::sync::Arc;
use glam::{Mat4, Quat, Vec3};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_task::{BuildTask, WriteFileTask};
use crate::data::skeleton_data::{BoneData, SkeletonData};

pub fn write_bones_data(
    dispatcher: Arc<Dispatcher>,
    paths: &AlpacaPaths,
    name: String,
    collection: &Node,
) -> Result<()> {
    let Some(skeleton_node) = collection.children().find(|child| child.name() == Some(&name)) else {
        bail!("Failed to find skeleton node. Searched for {}", name);
    };

    let Some(root_node) = skeleton_node.children().find(|child| child.name() == Some("root")) else {
        bail!("Failed to find root node");
    };

    let mut bones = Vec::new();
    collect_bone_nodes(&root_node, &mut bones);

    let parent_indices = bones.iter().map(|joint| {
        bones.iter()
            .position(|parent| {
                parent.children().any(|child| child.index() == joint.index())
            })
            .map(|index| index as i32)
            .unwrap_or(-1)
    }).collect::<Vec<_>>();

    let mut world_matrices = vec![Mat4::IDENTITY; bones.len()];

    for (i, joint) in bones.iter().enumerate() {
        let (translation, rotation, scale) = joint.transform().decomposed();
        let local = Mat4::from_scale_rotation_translation(
            Vec3::from(scale),
            Quat::from_array(rotation),
            Vec3::from(translation),
        );

        let parent_world = if parent_indices[i] >= 0 {
            world_matrices[parent_indices[i] as usize]
        } else {
            Mat4::IDENTITY
        };

        world_matrices[i] = parent_world * local;
    }

    let mut bones_data = Vec::with_capacity(bones.len());

    for i in 0..bones.len() {
        let bone = &bones[i];

        let Some(bone_name) = bone.name() else {
            bail!("Failed to extract bone name from node. All bones must have names! Skeleton: {}", name)
        };
        let parent_index = parent_indices[i];
        let inverse_bind_matrix = world_matrices[i].inverse().to_cols_array_2d();

        bones_data.push(BoneData {
            name: bone_name.to_string(),
            parent_index,

            inverse_bind_matrix,
        })
    }

    let path = paths.target.join(&name).with_extension("SKELETON");
    let skeleton_data = SkeletonData {
        name,

        bones: bones_data,
    };

    let skeleton_bytes = to_bytes::<Error>(&skeleton_data)?.into_vec();

    dispatcher.clone().dispatch(BuildTask::WriteFile(WriteFileTask {
        target_path: path,

        data: skeleton_bytes,
    }));

    Ok(())
}

fn collect_bone_nodes<'a>(node: &Node<'a>, result: &mut Vec<Node<'a>>) {
    result.push(node.clone());

    for child in node.children() {
        collect_bone_nodes(&child, result);
    }
}
