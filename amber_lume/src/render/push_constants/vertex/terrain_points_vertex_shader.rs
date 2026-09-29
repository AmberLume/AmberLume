use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainPointsVertexShader {
    camera_buffer_device_address: DeviceAddress,
    chunk_buffer_device_address: DeviceAddress,
    vertex_position_buffer_device_address: DeviceAddress,
    mesh_buffer_device_address: DeviceAddress,
    submesh_buffer_device_address: DeviceAddress,

    node_count: u32,
    point_size: f32,
    viewport_height: f32,

    _pad0: u32,
}

impl TerrainPointsVertexShader {
    pub fn create(
        camera_buffer: BufferRange,
        chunk_buffer: BufferRange,
        vertex_position_buffer: BufferRange,
        mesh_buffer: BufferRange,
        submesh_buffer: BufferRange,
        node_count: u32,
        point_size: f32,
        viewport_height: f32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            chunk_buffer_device_address: chunk_buffer.device_address,
            vertex_position_buffer_device_address: vertex_position_buffer.device_address,
            mesh_buffer_device_address: mesh_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,

            node_count,
            point_size,
            viewport_height,

            _pad0: 0,
        }
    }
}

impl VertexShader for TerrainPointsVertexShader {
    const SHADER: ShaderResource = shaders::terrain_points::TERRAIN_POINTS_VERT;
}
