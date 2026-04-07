use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinningInstanceGPU {
    pub animation_id: u32,
    pub skeleton_id: u32,
    pub bone_transform_offset: u32,
    pub time: f32,
    
    pub previous_animation_id: u32,
    pub previous_time: f32,
    pub blend_factor: f32,

    _pad0: u32,
}

impl SkinningInstanceGPU {
    pub fn new(
        animation_id: u32, 
        skeleton_id: u32,
        bone_transform_offset: u32,
        time: f32,
        previous_animation_id: u32,
        previous_time: f32,
        blend_factor: f32,
    ) -> Self {
        SkinningInstanceGPU {
            animation_id,
            skeleton_id,
            bone_transform_offset,
            time,

            previous_animation_id,
            previous_time,
            blend_factor,
            
            _pad0: 0,
        }
    }
}

pub fn create_skinning_instance_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<SkinningInstanceGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "skinning_instance",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::CpuToGpu,
    )
}
