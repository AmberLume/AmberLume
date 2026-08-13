use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct ReadbackHeader {
    pub written: u32,

    pub _pad0: [u32; 3],
}
