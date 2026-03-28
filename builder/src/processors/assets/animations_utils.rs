use std::collections::HashMap;
use crate::dispatcher::Dispatcher;
use anyhow::Result;
use gltf::{buffer, Animation};
use std::sync::Arc;
use gltf::animation::util::ReadOutputs;
use rkyv::rancor::Error;
use rkyv::to_bytes;
use crate::build_target::BuildTarget;
use crate::build_task::BuildTask;
use crate::data::animation_data::{AnimationData, BoneChannel};
use crate::data::skeleton_data::SkeletonData;
use crate::processors::utils::resource_key;

pub fn write_animation_data(
    dispatcher: Arc<Dispatcher>,
    build_target: &BuildTarget,
    skeleton_data: &SkeletonData,
    animation: &Animation,
    bin: Option<&[u8]>,
) -> Result<()> {
    let name = animation.name().unwrap().to_string();

    let name_to_bone = skeleton_data.bones.iter()
        .enumerate()
        .map(|(i, b)| (b.name.as_str(), i))
        .collect::<HashMap<&str, usize>>();

    let mut channels = vec![BoneChannel::default(); skeleton_data.bones.len()];

    for channel in animation.channels() {
        let bone_node = channel.target().node();
        let bone_name = bone_node.name().unwrap();
        let bone_index = name_to_bone[bone_name];

        let reader = channel.reader(|buffer| match buffer.source() {
            buffer::Source::Bin => None,
            buffer::Source::Uri(_) => bin,
        });

        let timestamps = reader.read_inputs().unwrap().collect::<Vec<_>>();

        match reader.read_outputs().unwrap() {
            ReadOutputs::Translations(translations) => {
                channels[bone_index].positions = timestamps.into_iter()
                    .zip(translations)
                    .collect()
            }
            ReadOutputs::Rotations(rotations) => {
                channels[bone_index].rotations = timestamps.into_iter()
                    .zip(rotations.into_f32())
                    .collect()
            }
            ReadOutputs::Scales(scales) => {
                channels[bone_index].scales = timestamps.into_iter()
                    .zip(scales)
                    .collect()
            }
            _ => {}
        }

    }

    let duration = max_duration(&channels, &|channel| &channel.positions)
        .max(max_duration(&channels, &|channel| &channel.rotations))
        .max(max_duration(&channels, &|channel| &channel.scales));

    let resource_key = resource_key(build_target, &name, "ANIMATION");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&AnimationData {
            name,

            duration,

            channels,
        })?.to_vec(),
    ));

    Ok(())
}

fn max_duration<T>(channels: &Vec<BoneChannel>, transforms: &dyn Fn(&BoneChannel) -> &Vec<(f32, T)>) -> f32 {
    channels.iter()
        .flat_map(|channel| {
            transforms(channel).last().map(|(time, _)| *time)
        })
        .fold(0.0_f32, f32::max)
}
