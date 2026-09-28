use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainPointsFragmentShader;

impl FragmentShader for TerrainPointsFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::terrain_points::TERRAIN_POINTS_FRAG);
}
