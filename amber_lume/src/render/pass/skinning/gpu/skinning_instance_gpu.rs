use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::{BufferRange, GpuSize};
use gpu_data::SubmeshGPU;
use crate::render::pass::skinning::gpu::skinning_pose_gpu::SkinningPoseGPU;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinningInstanceGPU {
    pub entity_index: u32,
    pub mesh_id: u32,
    pub skeleton_id: u32,
    pub bone_transform_offset: u32,

    pub previous_bone_transform_offset: u32,
    pub pose: SkinningPoseGPU,
    pub previous_pose: SkinningPoseGPU,
    _pad0: u32,

    pub submesh_buffer_device_address: DeviceAddress,
    _pad1: [u32; 2],
}

impl SkinningInstanceGPU {
    pub fn new(
        entity_index: u32,
        mesh_id: u32,
        skeleton_id: u32,
        bone_transform_offset: u32,
        previous_bone_transform_offset: u32,
        pose: SkinningPoseGPU,
        previous_pose: SkinningPoseGPU,
        submesh_offset: u32,
        skinned_submesh_offset: u32,
        skinned_submesh: BufferRange,
    ) -> Self {
        SkinningInstanceGPU {
            entity_index,
            mesh_id,
            skeleton_id,
            bone_transform_offset,

            previous_bone_transform_offset,
            pose,
            previous_pose,
            _pad0: 0,

            submesh_buffer_device_address: skinned_submesh.device_address
                .wrapping_add_signed((skinned_submesh_offset as i64 - submesh_offset as i64) * SubmeshGPU::SIZE as i64),
            _pad1: [0; 2],
        }
    }
}
