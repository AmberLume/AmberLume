#[repr(C)]
pub struct Entity {
    pub transform: [[f32; 4]; 4],
    pub mesh_id: u32,
    pub material_id: u32,
    pub _padding: [u32; 2],
}
