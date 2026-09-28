use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::render::push_constants::vertex_shader::VertexShader;

#[repr(C)]
pub struct PushConstants<V: VertexShader, F: FragmentShader> {
    pub vertex: V,
    pub fragment: F,
}
