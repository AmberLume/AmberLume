use bytemuck::{Pod, Zeroable};


#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub struct SdsmResultGPU {
    pub max_depth_bits: u32,
}
