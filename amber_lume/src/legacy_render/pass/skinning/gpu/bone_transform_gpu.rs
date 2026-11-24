use bytemuck::{Pod, Zeroable};
use math::Mat4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct BoneTransformGPU {
    pub transform: Mat4,
}
