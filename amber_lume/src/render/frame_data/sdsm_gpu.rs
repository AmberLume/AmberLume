use bytemuck::{Pod, Zeroable};


#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SdsmResultGPU {
    pub farthest_depth_bits: u32,
}

impl Default for SdsmResultGPU {
    fn default() -> Self {
        Self { farthest_depth_bits: 1.0f32.to_bits() }
    }
}
