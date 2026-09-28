use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct EnvironmentFragmentShader {
    pub scene_buffer_device_address: DeviceAddress,
    pub camera_buffer_device_address: DeviceAddress,
}

impl FragmentShader for EnvironmentFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::environment::ENVIRONMENT_FRAG);
}

impl EnvironmentFragmentShader {
    pub fn create(scene_buffer: BufferRange, camera_buffer: BufferRange) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            camera_buffer_device_address: camera_buffer.device_address,
        }
    }
}
