use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentEntityIdFragmentShader;

impl FragmentShader for TransparentEntityIdFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::transparent_entity_id::TRANSPARENT_ENTITY_ID_FRAG);
}
