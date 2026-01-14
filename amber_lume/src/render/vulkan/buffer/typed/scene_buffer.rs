use crate::render::vulkan::device_context::DeviceContext;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gpu_allocator::MemoryLocation;
use crate::render::vulkan::buffer::buffer::Buffer;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneGpuData {
    pub projection_matrix: [[f32; 4]; 4],

    pub indirect_buffer_device_address: u64,
    pub collider_indirect_buffer_device_address: u64,
    pub draw_count_buffer_device_address: u64,

    pub index_buffer_device_address: u64,
    pub vertex_buffer_device_address: u64,

    pub ui_index_buffer_device_address: u64,
    pub ui_vertex_buffer_device_address: u64,

    pub entity_buffer_device_address: u64,

    pub collider_buffer_device_address: u64,

    pub model_buffer_device_address: u64,
    pub model_availability_buffer_device_address: u64,

    pub material_buffer_device_address: u64,
    pub material_availability_buffer_device_address: u64,

    pub image_availability_buffer_device_address: u64,

    pub primitive_buffer_device_address: u64,
    
    pub draw_buffer_device_address: u64,
}

impl SceneGpuData {
    pub fn create(
        projection_matrix: Mat4,
        buffer_manager: &BufferManager,
    ) -> Self {
        Self {
            projection_matrix: projection_matrix.to_cols_array_2d(),

            indirect_buffer_device_address: buffer_manager.indirect_buffer.device_address.unwrap(),
            collider_indirect_buffer_device_address: buffer_manager.collider_indirect_buffer.device_address.unwrap(),
            draw_count_buffer_device_address: buffer_manager.draw_count_buffer.device_address.unwrap(),

            index_buffer_device_address: buffer_manager.index_buffer.device_address.unwrap(),
            vertex_buffer_device_address: buffer_manager.vertex_buffer.device_address.unwrap(),

            ui_index_buffer_device_address: buffer_manager.ui_index_buffer.device_address.unwrap(),
            ui_vertex_buffer_device_address: buffer_manager.ui_vertex_buffer.device_address.unwrap(),

            entity_buffer_device_address: buffer_manager.entity_buffer.device_address.unwrap(),

            collider_buffer_device_address: buffer_manager.collider_buffer.device_address.unwrap(),

            model_buffer_device_address: buffer_manager.model_buffer.device_address.unwrap(),
            model_availability_buffer_device_address: buffer_manager.model_availability_buffer.device_address.unwrap(),

            material_buffer_device_address: buffer_manager.material_buffer.device_address.unwrap(),
            material_availability_buffer_device_address: buffer_manager.material_availability_buffer.device_address.unwrap(),

            image_availability_buffer_device_address: buffer_manager.image_availability_buffer.device_address.unwrap(),

            primitive_buffer_device_address: buffer_manager.primitive_buffer.device_address.unwrap(),

            draw_buffer_device_address: buffer_manager.draw_buffer.device_address.unwrap(),
        }
    }
}

pub fn create_scene_buffer(
    device_context: &mut DeviceContext,
) -> Result<Buffer> {
    Buffer::create(
        device_context,
        "scene_buffer",
        1,
        size_of::<SceneGpuData>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::CpuToGpu,
    )
}
