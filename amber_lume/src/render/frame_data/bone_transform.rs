use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct BoneTransformGPU {
    pub transform: [[f32; 4]; 4],
}
