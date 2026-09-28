use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct PhysicsDebugFragmentShader;

impl FragmentShader for PhysicsDebugFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::physics_debug::PHYSICS_DEBUG_FRAG);
}
