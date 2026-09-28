use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct UpsampleFragmentShader {
    pub src_texture: u32,
}

impl FragmentShader for UpsampleFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::bloom::UPSAMPLE_FRAG);
}

impl UpsampleFragmentShader {
    pub fn create(src_texture: u32) -> Self {
        Self {
            src_texture,
        }
    }
}
