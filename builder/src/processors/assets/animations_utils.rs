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
use crate::data::animation_data::{AnimationData, AnimationKeyframe};
use crate::data::skeleton_data::SkeletonData;
use crate::processors::utils::resource_key;

#[derive(Clone)]
struct BoneChannel {
    positions: Vec<(f32, [f32; 3])>,
    rotations: Vec<(f32, [f32; 4])>,
    scales: Vec<(f32, [f32; 3])>,
}

impl Default for BoneChannel {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            rotations: Vec::new(),
            scales: Vec::new(),
        }
    }
}

static ANIMATION_FPS: f32 = 30.0;

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

    let duration = max_duration(&channels, &|c| &c.positions)
        .max(max_duration(&channels, &|c| &c.rotations))
        .max(max_duration(&channels, &|c| &c.scales));

    let bone_count = channels.len() as u32;
    let frame_count = (duration * ANIMATION_FPS).ceil() as u32 + 1;
    let keyframes = bake_keyframes(&channels, ANIMATION_FPS, frame_count);

    let resource_key = resource_key(build_target, &name, "ANIMATION");
    dispatcher.dispatch(BuildTask::archive(
        build_target,
        &resource_key,
        to_bytes::<Error>(&AnimationData {
            name,
            duration,
            fps: ANIMATION_FPS,
            bone_count,
            frame_count,
            keyframes,
        })?.to_vec(),
    ));

    Ok(())
}

fn bake_keyframes(channels: &[BoneChannel], fps: f32, frame_count: u32) -> Vec<AnimationKeyframe> {
    let mut keyframes = Vec::with_capacity(channels.len() * frame_count as usize);

    for channel in channels {
        for frame in 0..frame_count {
            let t = frame as f32 / fps;

            keyframes.push(AnimationKeyframe {
                translation: sample_vec3(&channel.positions, t),
                rotation: sample_quaternion(&channel.rotations, t),
                scale: sample_scale(&channel.scales, t),
            });
        }
    }

    keyframes
}

fn max_duration<T>(channels: &Vec<BoneChannel>, transforms: &dyn Fn(&BoneChannel) -> &Vec<(f32, T)>) -> f32 {
    channels.iter()
        .flat_map(|channel| {
            transforms(channel).last().map(|(time, _)| *time)
        })
        .fold(0.0_f32, f32::max)
}

fn sample_vec3(samples: &[(f32, [f32; 3])], t: f32) -> [f32; 3] {
    if samples.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    if samples.len() == 1 || t <= samples[0].0 {
        return samples[0].1;
    }
    if t >= samples.last().unwrap().0 {
        return samples.last().unwrap().1;
    }

    let i = samples.partition_point(|(time, _)| *time <= t) - 1;

    let (t0, a) = samples[i];
    let (t1, b) = samples[i + 1];
    let alpha = (t - t0) / (t1 - t0);

    [
        a[0] + (b[0] - a[0]) * alpha,
        a[1] + (b[1] - a[1]) * alpha,
        a[2] + (b[2] - a[2]) * alpha,
    ]
}

fn sample_scale(samples: &[(f32, [f32; 3])], t: f32) -> [f32; 3] {
    if samples.is_empty() { return [1.0, 1.0, 1.0]; }

    sample_vec3(samples, t)
}

fn sample_quaternion(samples: &[(f32, [f32; 4])], t: f32) -> [f32; 4] {
    if samples.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    if samples.len() == 1 || t <= samples[0].0 {
        return samples[0].1;
    }
    if t >= samples.last().unwrap().0 {
        return samples.last().unwrap().1;
    }

    let i = samples.partition_point(|(time, _)| *time <= t) - 1;

    let (t0, a) = samples[i];
    let (t1, b) = samples[i + 1];
    let alpha = (t - t0) / (t1 - t0);

    slerp(a, b, alpha)
}

fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let b = if dot < 0.0 {
        dot = -dot;
        [-b[0], -b[1], -b[2], -b[3]]
    } else {
        b
    };

    if dot > 0.9995 {
        let r = [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ];
        let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + r[3] * r[3]).sqrt();

        return [r[0] / len, r[1] / len, r[2] / len, r[3] / len];
    }

    let theta_0 = dot.acos();
    let theta = theta_0 * t;
    let s0 = (theta_0 - theta).sin() / theta_0.sin();
    let s1 = theta.sin() / theta_0.sin();

    [
        a[0] * s0 + b[0] * s1,
        a[1] * s0 + b[1] * s1,
        a[2] * s0 + b[2] * s1,
        a[3] * s0 + b[3] * s1,
    ]
}
