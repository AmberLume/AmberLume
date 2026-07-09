use bytemuck::{Pod, Zeroable};
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTAOPushConstants {
    pub inverse_view_projection: [[f32; 4]; 4],

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub ao_storage_id: u32,
    pub width: u32,
    pub height: u32,
    pub tlas_descriptor_id: u32,

    pub ao_radius: f32,
    pub sample_count: u32,
    pub ao_power: f32,
    pub frame_number: u32,

    _pad0: [u32; 6],
}

impl RTAOPushConstants {
    pub fn create(
        view_projection: &ViewProjectionMatrix,
        depth_descriptor_id: u32,
        normal_descriptor_id: u32,
        ao_storage_id: u32,
        width: u32,
        height: u32,
        tlas_descriptor_id: u32,
        ao_radius: f32,
        sample_count: u32,
        ao_power: f32,
        frame_number: u32,
    ) -> Self {
        Self {
            inverse_view_projection: view_projection.inverted().value.to_cols_array_2d(),

            depth_descriptor_id,
            normal_descriptor_id,
            ao_storage_id,
            width,
            height,
            tlas_descriptor_id,

            ao_radius,
            sample_count,
            ao_power,
            frame_number,

            _pad0: [0; 6],
        }
    }
}
