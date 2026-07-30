#[repr(C)]
pub struct IndirectGPU {
    pub index_count: u32,
    pub instance_count: u32,
    pub index_offset: u32,
    pub vertex_offset: i32,
    pub instance_offset: u32,
}
