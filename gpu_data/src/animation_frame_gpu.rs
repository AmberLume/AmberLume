use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct AnimationFrameGPU {
    pub translation: [f32; 3],
    _pad0: u32,
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    _pad1: u32,
}

impl AnimationFrameGPU {
    pub fn create(
        translation: [f32; 3],
        rotation: [f32; 4],
        scale: [f32; 3],
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
