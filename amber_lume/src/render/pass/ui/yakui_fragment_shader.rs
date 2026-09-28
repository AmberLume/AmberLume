use crate::data::resource_handle::ShaderResource;
use crate::render::push_constants::fragment_shader::FragmentShader;
use crate::resource_manifest::shaders;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct YakuiFragmentShader {
    pub texture_index: u32,
    pub render_mode: u32,
}

impl FragmentShader for YakuiFragmentShader {
    const SHADER: Option<ShaderResource> = Some(shaders::yakui::YAKUI_FRAG);
}

impl YakuiFragmentShader {
    pub fn create(texture_index: u32, render_mode: u32) -> Self {
        Self {
            texture_index,
            render_mode,
        }
    }
}
