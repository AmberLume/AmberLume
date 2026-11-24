use bytemuck::{Pod, Zeroable};
use math::Vec4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityOutlineGPU {
    pub outline: Vec4,
}

impl EntityOutlineGPU {
    pub fn create(outline: Vec4) -> Self {
        Self { outline }
    }
}
