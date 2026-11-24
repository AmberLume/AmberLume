use bytemuck::{Pod, Zeroable};
use math::{Quat, Vec3};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct AnimationFrameGPU {
    pub translation: Vec3,
    _pad0: u32,
    pub rotation: Quat,
    pub scale: Vec3,
    _pad1: u32,
}

impl AnimationFrameGPU {
    pub fn create(
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> Self {
        Self {
            translation,
            _pad0: 0,
            rotation,
            scale,
            _pad1: 0,
        }
    }
}
