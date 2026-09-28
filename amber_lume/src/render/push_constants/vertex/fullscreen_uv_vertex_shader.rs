use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct FullscreenUvVertexShader;

impl VertexShader for FullscreenUvVertexShader {
    const SHADER: ShaderResource = shaders::fullscreen_uv::FULLSCREEN_UV_VERT;
}
