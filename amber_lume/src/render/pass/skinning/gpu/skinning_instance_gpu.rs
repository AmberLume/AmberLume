use bytemuck::{Pod, Zeroable};
use crate::render::pass::skinning::gpu::skinning_pose_gpu::SkinningPoseGPU;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinningInstanceGPU {
    pub mesh_id: u32,
    pub skeleton_id: u32,
    pub bone_transform_offset: u32,
    pub bounds_index: u32,

    pub pose: SkinningPoseGPU,
}

impl SkinningInstanceGPU {
    pub fn new(
        mesh_id: u32,
        skeleton_id: u32,
        bone_transform_offset: u32,
        bounds_index: u32,
        pose: SkinningPoseGPU,
    ) -> Self {
        Self {
            mesh_id,
            skeleton_id,
            bone_transform_offset,
            bounds_index,

            pose,
        }
    }
}
