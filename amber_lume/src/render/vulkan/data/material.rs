#[repr(C)]
pub struct Material {
    pub albedo_index: u32,
    pub normal_index: u32,
    pub metallic_roughness_index: u32,
    pub _padding: u32,
}
