use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkeletonGPU {
    pub offset: u32,
    pub count: u32,

    _pad0: [u32; 2],
}

impl SkeletonGPU {
    pub fn create(offset: u32, count: u32) -> Self {
        Self {
            offset,
            count,

            _pad0: [0; 2],
        }
    }
}
