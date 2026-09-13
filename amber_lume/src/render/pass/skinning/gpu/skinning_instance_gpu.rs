use bytemuck::{Pod, Zeroable};
use crate::render::pass::skinning::gpu::skinning_pose_gpu::SkinningPoseGPU;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinningInstanceGPU {
    pub skeleton_id: u32,
    pub bone_transform_offset: u32,
    pub previous_bone_transform_offset: u32,

    pub pose: SkinningPoseGPU,
    pub previous_pose: SkinningPoseGPU,

    _pad0: [u32; 3],
}

impl SkinningInstanceGPU {
    pub fn new(
        skeleton_id: u32,
        bone_transform_offset: u32,
        previous_bone_transform_offset: u32,
        pose: SkinningPoseGPU,
        previous_pose: SkinningPoseGPU,
    ) -> Self {
        SkinningInstanceGPU {
            skeleton_id,
            bone_transform_offset,
            previous_bone_transform_offset,

            pose,
            previous_pose,

            _pad0: [0; 3],
        }
    }
}
