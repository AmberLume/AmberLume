use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct BrdfLutFragmentShader;

impl FragmentShader for BrdfLutFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::brdf_lut::BRDF_LUT_FRAG);
}
