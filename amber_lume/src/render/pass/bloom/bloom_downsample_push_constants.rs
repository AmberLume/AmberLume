use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct BloomDownsamplePushConstants {
    pub source_descriptor_id: u32,
    pub karis_average: u32,
    pub threshold: f32,

    _pad0: [u32; 29],
}

impl BloomDownsamplePushConstants {
    pub fn create(source_descriptor_id: u32, karis_average: u32, threshold: f32) -> Self {
        Self {
            source_descriptor_id,
            karis_average,
            threshold,

            _pad0: [0; 29],
        }
    }
}
