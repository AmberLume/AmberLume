use crate::data::resource_handle::ShaderResource;
use bytemuck::Pod;

pub trait FragmentShader: Pod {
    const SHADER: Option<ShaderResource>;
}

impl FragmentShader for () {
    const SHADER: Option<ShaderResource> = None;
}
