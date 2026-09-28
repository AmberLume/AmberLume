use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;
use index_allocator::ResourceId;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct MainFragmentShader {
    pub scene_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub picked_entity_buffer_device_address: DeviceAddress,

    pub shadow_factor_descriptor_id: u32,
    pub shadow_enabled: u32,
    pub shadow_colored: u32,

    pub gtao_descriptor_id: u32,
    pub ao_enabled: u32,

    pub sh_descriptor_id: u32,
    pub brdf_lut_descriptor_id: u32,

    pub pick_x: u32,
    pub pick_y: u32,

    _pad0: u32,
}

impl FragmentShader for MainFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::main::MAIN_FRAG);
}

impl MainFragmentShader {
    pub fn create(
        scene_buffer: BufferRange,
        material_buffer: BufferRange,
        picked_entity: BufferRange,
        shadow_factor_descriptor_id: ResourceId,
        shadow_enabled: u32,
        shadow_colored: u32,
        gtao_descriptor_id: ResourceId,
        ao_enabled: u32,
        sh_descriptor_id: ResourceId,
        brdf_lut_descriptor_id: ResourceId,
        pick_x: u32,
        pick_y: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            material_buffer_device_address: material_buffer.device_address,
            picked_entity_buffer_device_address: picked_entity.device_address,

            shadow_factor_descriptor_id: shadow_factor_descriptor_id.inner,
            shadow_enabled,
            shadow_colored,

            gtao_descriptor_id: gtao_descriptor_id.inner,
            ao_enabled,

            sh_descriptor_id: sh_descriptor_id.inner,
            brdf_lut_descriptor_id: brdf_lut_descriptor_id.inner,

            pick_x,
            pick_y,

            _pad0: 0,
        }
    }
}
