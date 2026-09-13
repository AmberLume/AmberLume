use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::{BufferRange, GpuSize};
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinCacheInstanceGPU {
    pub entity_index: u32,
    pub vertex_offset: u32,
    pub vertex_attribute_offset: u32,
    pub vertex_skin_offset: u32,

    pub bone_transform_offset: u32,
    pub skin_cache_offset: u32,

    pub vertex_buffer_device_address: DeviceAddress,
    pub vertex_attribute_buffer_device_address: DeviceAddress,

    _pad0: [u32; 2],
}

impl SkinCacheInstanceGPU {
    pub fn new(
        entity_index: u32,
        vertex_offset: u32,
        vertex_attribute_offset: u32,
        vertex_skin_offset: u32,
        bone_transform_offset: u32,
        skin_cache_offset: u32,
        skin_cache_vertex: BufferRange,
        skin_cache_vertex_attribute: BufferRange,
    ) -> Self {
        Self {
            entity_index,
            vertex_offset,
            vertex_attribute_offset,
            vertex_skin_offset,

            bone_transform_offset,
            skin_cache_offset,

            vertex_buffer_device_address: skin_cache_vertex.device_address
                .wrapping_add_signed((skin_cache_offset as i64 - vertex_offset as i64) * MeshVertexGPU::SIZE as i64),
            vertex_attribute_buffer_device_address: skin_cache_vertex_attribute.device_address
                .wrapping_add_signed((skin_cache_offset as i64 - vertex_attribute_offset as i64) * MeshVertexAttributeGPU::SIZE as i64),

            _pad0: [0; 2],
        }
    }
}
