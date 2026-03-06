use crate::render::vulkan::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::vulkan::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::vulkan::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MaterialGpuData {
    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metalic_factor: f32,

    pub color_texture_index: u32,
    pub normal_texture_index: u32,
    pub occlusion_roughness_metallic_texture_index: u32,

    _pad0: [u32; 3],
}

impl MaterialGpuData {
    pub fn create(
        base_color_factor: [f32; 4],
        roughness_factor: f32,
        metalic_factor: f32,
        color_texture_index: u32,
        normal_texture_index: u32,
        occlusion_roughness_metallic_texture_index: u32,
    ) -> Self {
        Self {
            base_color_factor,
            roughness_factor,
            metalic_factor,

            color_texture_index,
            normal_texture_index,
            occlusion_roughness_metallic_texture_index,

            _pad0: [0; 3],
        }
    }
}

pub fn create_material_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<MaterialGpuData>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "material_buffer",
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
