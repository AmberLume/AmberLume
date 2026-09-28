use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct MeshPositionVertexShader {
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub shadow_cascades_buffer_device_address: DeviceAddress,
}

impl MeshPositionVertexShader {
    pub fn create(
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        shadow_cascades_buffer: BufferRange,
    ) -> Self {
        Self {
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            shadow_cascades_buffer_device_address: shadow_cascades_buffer.device_address,
        }
    }
}

impl VertexShader for MeshPositionVertexShader {
    const SHADER: ShaderResource = shaders::mesh_position::MESH_POSITION_VERT;
}
