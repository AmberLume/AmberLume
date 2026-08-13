use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshGPU {
    pub submesh_offset: u32,
    pub submesh_count: u32,
    _pad0: [u32; 2],
}

impl MeshGPU {
    pub fn create(submesh_offset: u32, submesh_count: u32) -> Self {
        Self {
            submesh_offset,
            submesh_count,
            _pad0: [0; 2],
        }
    }
}
