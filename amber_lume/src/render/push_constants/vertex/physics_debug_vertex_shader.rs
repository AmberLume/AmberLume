use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct PhysicsDebugVertexShader {
    pub camera_buffer_device_address: DeviceAddress,
    pub physics_debug_vertex_buffer_device_address: DeviceAddress,
}

impl PhysicsDebugVertexShader {
    pub fn create(
        camera_buffer: BufferRange,
        physics_debug_vertex_buffer: BufferRange,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            physics_debug_vertex_buffer_device_address: physics_debug_vertex_buffer.device_address,
        }
    }
}

impl VertexShader for PhysicsDebugVertexShader {
    const SHADER: ShaderResource = shaders::physics_debug::PHYSICS_DEBUG_VERT;
}
