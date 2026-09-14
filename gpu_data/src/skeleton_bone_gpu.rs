use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkeletonBoneGPU {
    pub parent: i32,

    _pad0: [u32; 3],
}

impl SkeletonBoneGPU {
    pub fn create(parent: i32) -> Self {
        Self {
            parent,

            _pad0: [0; 3],
        }
    }
}
