use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MeshGPU {
    pub submesh_offset: u32,
    pub submesh_count: u32,
    pub binding_offset: u32,
    pub vertex_slice_stride: u32,
    pub vertex_slice_count: u32,
}

impl MeshGPU {
    pub fn create(
        submesh_offset: u32,
        submesh_count: u32,
        binding_offset: u32,
        vertex_slice_stride: u32,
        vertex_slice_count: u32,
    ) -> Self {
        Self {
            submesh_offset,
            submesh_count,
            binding_offset,
            vertex_slice_stride,
            vertex_slice_count,
        }
    }
}
