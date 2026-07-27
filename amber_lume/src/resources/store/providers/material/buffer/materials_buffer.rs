use crate::render::factories::buffer::builder::buffer_builder::BufferBuilder;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use anyhow::Result;
use ash::vk::BufferUsageFlags;
use bytemuck::{Pod, Zeroable};
use gpu_allocator::MemoryLocation;
use crate::data::alpha_mode::AlphaMode;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MaterialGPU {
    pub base_color_factor: [f32; 4],
    pub roughness_factor: f32,
    pub metallic_factor: f32,

    pub color_texture_index: u32,
    pub normal_texture_index: u32,
    pub occlusion_roughness_metallic_texture_index: u32,

    pub flags: u32,
    pub alpha_cutoff: f32,

    _pad0: u32,
}

impl MaterialGPU {
    pub const FLAG_ALPHA_OPAQUE: u32 = 1 << 0;
    pub const FLAG_ALPHA_MASK: u32 = 1 << 1;
    pub const FLAG_ALPHA_BLEND: u32 = 1 << 2;

    pub const ALPHA_MODE_BITS: u32 = Self::FLAG_ALPHA_OPAQUE | Self::FLAG_ALPHA_MASK | Self::FLAG_ALPHA_BLEND;

    pub const DEFAULT: Self = Self {
        base_color_factor: [1.0, 0.0, 1.0, 1.0],
        roughness_factor: 1.0,
        metallic_factor: 1.0,

        color_texture_index: 0,
        normal_texture_index: 0,
        occlusion_roughness_metallic_texture_index: 0,

        flags: Self::FLAG_ALPHA_OPAQUE,
        alpha_cutoff: AlphaMode::DEFAULT_CUTOFF,

        _pad0: 0,
    };

    pub fn create(
        base_color_factor: [f32; 4],
        roughness_factor: f32,
        metallic_factor: f32,
        alpha_mode: AlphaMode,
        alpha_cutoff: f32,
        color_texture_index: u32,
        normal_texture_index: u32,
        occlusion_roughness_metallic_texture_index: u32,
    ) -> Self {
        let alpha_flag = match alpha_mode {
            AlphaMode::Opaque => Self::FLAG_ALPHA_OPAQUE,
            AlphaMode::Mask => Self::FLAG_ALPHA_MASK,
            AlphaMode::Blend => Self::FLAG_ALPHA_BLEND,
        };

        Self {
            base_color_factor,
            roughness_factor,
            metallic_factor,

            color_texture_index,
            normal_texture_index,
            occlusion_roughness_metallic_texture_index,

            flags: alpha_flag,
            alpha_cutoff,

            _pad0: 0,
        }
    }
}

pub fn create_materials_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<SliceBuffer<MaterialGPU>> {
    BufferBuilder::slice(capacity).build(
        buffer_factory,
        "materials",
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
