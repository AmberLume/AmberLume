use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DownsampleFragmentShader {
    pub src_texture: u32,
    pub karis: u32,
    pub threshold: f32,
}

impl FragmentShader for DownsampleFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::bloom::DOWNSAMPLE_FRAG);
}

impl DownsampleFragmentShader {
    pub fn create(src_texture: u32, karis: u32, threshold: f32) -> Self {
        Self {
            src_texture,
            karis,
            threshold,
        }
    }
}
