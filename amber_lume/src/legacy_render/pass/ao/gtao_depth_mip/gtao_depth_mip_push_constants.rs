use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GtaoDepthMipPushConstants {
    pub source_descriptor_id: u32,
    pub view_z_storage_id: u32,
    pub width: u32,
    pub height: u32,
    pub source_width: u32,
    pub source_height: u32,

    pub radius: f32,

    _pad0: [u32; 25],
}

impl GtaoDepthMipPushConstants {
    pub fn create(
        source_descriptor_id: u32,
        view_z_storage_id: u32,
        width: u32,
        height: u32,
        source_width: u32,
        source_height: u32,
        radius: f32,
    ) -> Self {
        Self {
            source_descriptor_id,
            view_z_storage_id,
            width,
            height,
            source_width,
            source_height,

            radius,

            _pad0: [0; 25],
        }
    }
}
