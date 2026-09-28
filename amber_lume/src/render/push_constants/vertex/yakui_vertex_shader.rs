use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct YakuiVertexShader {
    pub ui_vertex_buffer_device_address: DeviceAddress,
}

impl YakuiVertexShader {
    pub fn create(
        ui_vertex_buffer: BufferRange,
    ) -> Self {
        Self {
            ui_vertex_buffer_device_address: ui_vertex_buffer.device_address,
        }
    }
}

impl VertexShader for YakuiVertexShader {
    const SHADER: ShaderResource = shaders::yakui::YAKUI_VERT;
}
