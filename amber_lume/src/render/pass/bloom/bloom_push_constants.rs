use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BloomPushConstants {
    pub src_texture: u32,
    pub karis: u32,
    pub threshold: f32,
}
