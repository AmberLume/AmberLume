use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DebugLayerFragmentShader {
    pub camera_buffer_device_address: DeviceAddress,

    pub texture_index: u32,
    pub layer_kind: u32,
    pub shadow_colored: u32,

    pub denoise_history: f32,
}

impl FragmentShader for DebugLayerFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::debug_layer::DEBUG_LAYER_FRAG);
}

impl DebugLayerFragmentShader {
    pub fn create(
        camera_buffer: BufferRange,
        texture_index: u32,
        layer_kind: u32,
        shadow_colored: u32,
        denoise_history: f32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,

            texture_index,
            layer_kind,
            shadow_colored,

            denoise_history,
        }
    }
}
