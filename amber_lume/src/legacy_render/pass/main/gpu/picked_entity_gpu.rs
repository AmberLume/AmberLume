use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct PickedEntityGPU {
    pub id: u32,
}
