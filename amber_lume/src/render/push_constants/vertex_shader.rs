use crate::data::resource_handle::ShaderResource;
use bytemuck::Pod;

pub trait VertexShader: Pod {
    const SHADER: ShaderResource;
}
