use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct EntityOutlineGPU {
    pub outline: [f32; 4],
}

impl EntityOutlineGPU {
    pub fn create(outline: [f32; 4]) -> Self {
        Self { outline }
    }
}
