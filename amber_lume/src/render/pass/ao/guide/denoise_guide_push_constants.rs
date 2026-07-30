use bytemuck::{Pod, Zeroable};
use gpu::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DenoiseGuidePushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub camera_position: [f32; 4],

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub guide_storage_id: u32,
    pub width: u32,
    pub height: u32,

    _pad0: [u32; 7],
}

impl DenoiseGuidePushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        camera_position: [f32; 3],
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        guide_storage_id: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            camera_position: [camera_position[0], camera_position[1], camera_position[2], 0.0],

            depth_descriptor_id,
            normal_descriptor_id,
            guide_storage_id,
            width,
            height,

            _pad0: [0; 7],
        }
    }
}
