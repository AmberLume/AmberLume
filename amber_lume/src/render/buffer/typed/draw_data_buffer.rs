use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawDataGPU {
    pub entity_index: u32,
    pub submesh_index: u32,
    pub cascade_mask: u32,
    _pad0: u32,
}
