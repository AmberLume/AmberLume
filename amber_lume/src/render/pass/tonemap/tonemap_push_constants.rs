use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TonemapPushConstants {
    pub input_texture: u32,
    pub exposure: f32,
    pub hdr: u32,
    pub paper_white: f32,
    pub bloom_texture: u32,
    pub bloom_intensity: f32,
    pub sharpness: f32,

    _pad0: [u32; 25],
}

impl TonemapPushConstants {
    pub fn create(
        input_texture: u32,
        exposure: f32,
        hdr: u32,
        paper_white: f32,
        bloom_texture: u32,
        bloom_intensity: f32,
        sharpness: f32,
    ) -> Self {
        Self {
            input_texture,
            exposure,
            hdr,
            paper_white,
            bloom_texture,
            bloom_intensity,
            sharpness,

            _pad0: [0; 25],
        }
    }
}
