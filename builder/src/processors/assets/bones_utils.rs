use std::collections::HashMap;
use crate::dispatcher::Dispatcher;
use anyhow::Result;
use anyhow::bail;
use gltf::{Node, Skin};
use std::sync::Arc;
use glam::{Mat4, Quat, Vec3};
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use amber_lume::data::skeleton_data::{BoneData, SkeletonData};
use crate::processors::utils::resource_key;

pub fn write_bones_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    name: String,
    collection: &Node,
    skeletons: Vec<Skin>,
    bin: Option<&[u8]>,
) -> Result<SkeletonData> {
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

    let ibm_map: HashMap<usize, Mat4> = skeletons.first()
        .map(|skin| {
            let reader = skin.reader(|buffer| match buffer.source() {
                gltf::buffer::Source::Bin => None,
                gltf::buffer::Source::Uri(_) => bin,
            });

            let ibms: Vec<Mat4> = reader.read_inverse_bind_matrices()
                .map(|iter| iter.map(|m| Mat4::from_cols_array_2d(&m)).collect())
                .unwrap_or_default();

            skin.joints()
                .enumerate()
                .filter_map(|(i, joint)| ibms.get(i).map(|ibm| (joint.index(), *ibm)))
                .collect()
        })
        .unwrap_or_default();

    let mut bones_data = Vec::with_capacity(bones.len());

    for i in 0..bones.len() {
        let bone = &bones[i];

        let Some(bone_name) = bone.name() else {
            bail!("Failed to extract bone name from node. All bones must have names! Skeleton: {}", name)
        };

        let parent_index = parent_indices[i];

        let inverse_bind_matrix = if let Some(ibm) = ibm_map.get(&bone.index()) {
            ibm.to_cols_array_2d()
        } else {
            let mut world = Mat4::IDENTITY;
            let mut stack = vec![i];
            let mut idx = i;
            while parent_indices[idx] >= 0 {
                idx = parent_indices[idx] as usize;
                stack.push(idx);
            }
            for &j in stack.iter().rev() {
                let (t, r, s) = bones[j].transform().decomposed();
                let local = Mat4::from_scale_rotation_translation(Vec3::from(s), Quat::from_array(r), Vec3::from(t));
                world = world * local;
            }
            world.inverse().to_cols_array_2d()
        };

        bones_data.push(BoneData {
            name: bone_name.to_string(),
            parent_index,
            inverse_bind_matrix,
        });
    }

    let resource_key = resource_key(build_target, &name, "SKELETON");
    let skeleton_data = SkeletonData {
        name: name.clone(),

        bones: bones_data,
    };
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&skeleton_data)?.to_vec(),
    ));

    Ok(skeleton_data)
}

pub fn collect_bone_nodes<'a>(node: &Node<'a>, result: &mut Vec<Node<'a>>) {
    result.push(node.clone());

    let mut children = node.children().collect::<Vec<_>>();
    children.sort_by_key(|child| child.name().unwrap());

    for child in children {
        collect_bone_nodes(&child, result);
    }
}
