use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TonemapPushConstants {
    pub input_texture: u32,
    pub exposure: f32,
    pub hdr: u32,
    pub paper_white: f32,
}
