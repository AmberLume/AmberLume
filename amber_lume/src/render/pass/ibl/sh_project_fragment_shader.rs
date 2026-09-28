use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct ShProjectFragmentShader {
    pub scene_buffer_device_address: DeviceAddress,
}

impl FragmentShader for ShProjectFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::ibl::SH_PROJECT_FRAG);
}

impl ShProjectFragmentShader {
    pub fn create(scene_buffer: BufferRange) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
        }
    }
}
