use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SelectionFragmentShader {
    pub camera_buffer_device_address: DeviceAddress,
    pub entity_outline_buffer_device_address: DeviceAddress,

    pub entity_id_texel_scale: [f32; 2],

    pub entity_id_texture: u32,
    pub mask_texture: u32,

    pub radius: i32,
    pub mask_scale: i32,
}

impl FragmentShader for SelectionFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::selection::SELECTION_FRAG);
}

impl SelectionFragmentShader {
    pub fn create(
        camera_buffer: BufferRange,
        entity_outline_buffer: BufferRange,
        entity_id_texel_scale: [f32; 2],
        entity_id_texture: u32,
        mask_texture: u32,
        radius: i32,
        mask_scale: i32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            entity_outline_buffer_device_address: entity_outline_buffer.device_address,

            entity_id_texel_scale,

            entity_id_texture,
            mask_texture,

            radius,
            mask_scale,
        }
    }
}
