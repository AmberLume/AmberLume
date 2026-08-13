use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BloomPushConstants {
    pub src_texture: u32,
    pub karis: u32,
    pub threshold: f32,

    _pad0: [u32; 29],
}

impl BloomPushConstants {
    pub fn create(src_texture: u32, karis: u32, threshold: f32) -> Self {
        Self {
            src_texture,
            karis,
            threshold,

            _pad0: [0; 29],
        }
    }
}
