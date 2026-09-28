use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentFragmentShader {
    pub scene_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,

    pub sh_descriptor_id: u32,
    pub brdf_lut_descriptor_id: u32,
}

impl FragmentShader for TransparentFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::transparent::TRANSPARENT_FRAG);
}

impl TransparentFragmentShader {
    pub fn create(
        scene_buffer: BufferRange,
        material_buffer: BufferRange,
        sh_descriptor_id: u32,
        brdf_lut_descriptor_id: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            material_buffer_device_address: material_buffer.device_address,

            sh_descriptor_id,
            brdf_lut_descriptor_id,
        }
    }
}
