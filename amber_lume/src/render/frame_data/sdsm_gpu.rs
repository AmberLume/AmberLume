use bytemuck::{Pod, Zeroable};


#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone, Debug, Default)]
pub struct SdsmResultGPU {
    pub z_max_bits: u32,
}
