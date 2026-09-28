use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::vertex_shader::VertexShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct FullscreenPositionVertexShader;

impl VertexShader for FullscreenPositionVertexShader {
    const SHADER: ShaderResource = shaders::fullscreen_position::FULLSCREEN_POSITION_VERT;
}
