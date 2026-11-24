use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct DrawDataGPU {
    pub entity_index: u32,
    pub submesh_index: u32,
    pub previous_vertex_delta: i32,
    pub cascade_mask: u32,
    pub sort_key: f32,
}
