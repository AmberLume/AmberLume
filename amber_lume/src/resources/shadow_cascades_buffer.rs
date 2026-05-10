use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct ShadowCascadeGPU {
    pub light_space_view_projection: [[f32; 4]; 4],
    pub screen_to_light: [[f32; 4]; 4],
    pub split: f32,
    pub world_radius: f32,
    _pad0: [u32; 2],
}

impl Default for ShadowCascadeGPU {
    fn default() -> Self {
        Self {
            light_space_view_projection: [[0.0; 4]; 4],
            screen_to_light: [[0.0; 4]; 4],
            split: 0.0,
            world_radius: 0.0,

            _pad0: [0; 2],
        }
    }
}

pub fn create_shadow_cascades_buffer(
    buffer_factory: &ManagedBufferFactory,
    cascade_count: u32,
) -> Result<SliceBuffer<ShadowCascadeGPU>> {
    BufferBuilder::slice(cascade_count).build(
        buffer_factory,
        "shadow_cascades",
        BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::GpuOnly,
    )
}
