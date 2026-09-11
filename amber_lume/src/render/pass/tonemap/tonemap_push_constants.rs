use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TonemapPushConstants {
    pub input_texture: u32,
    pub exposure: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub offset: [f32; 3],
    pub gamma: [f32; 3],
    pub gain: [f32; 3],
    pub hdr: u32,
    pub paper_white: f32,
    pub display_peak: f32,
    pub bloom_texture: u32,
    pub bloom_intensity: f32,
    pub sharpness: f32,

    _pad0: [u32; 13],
}

impl TonemapPushConstants {
    pub fn create(
        input_texture: u32,
        exposure: f32,
        saturation: f32,
        contrast: f32,
        offset: [f32; 3],
        gamma: [f32; 3],
        gain: [f32; 3],
        hdr: u32,
        paper_white: f32,
        display_peak: f32,
        bloom_texture: u32,
        bloom_intensity: f32,
        sharpness: f32,
    ) -> Self {
        Self {
            input_texture,
            exposure,
            saturation,
            contrast,
            offset,
            gamma,
            gain,
            hdr,
            paper_white,
            display_peak,
            bloom_texture,
            bloom_intensity,
            sharpness,

            _pad0: [0; 13],
        }
    }
}
