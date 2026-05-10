use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use crate::utils::matrix_wrappers::ViewProjectionMatrix;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MainCameraGPU {
    pub view_projection: [[f32; 4]; 4],

    pub position: [f32; 3],
    _pad0: u32,

    pub near: f32,
    pub far: f32,
    _pad1: [u32; 2],
}

impl MainCameraGPU {
    pub fn new(
        view_projection: &ViewProjectionMatrix,
        position: Vec3,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            view_projection: view_projection.value.to_cols_array_2d(),

            position: position.to_array(),
            _pad0: 0,

            near,
            far,
            _pad1: [0; 2],
        }
    }
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneGPU {
    pub main_camera: MainCameraGPU,

    pub light_direction: [f32; 3],
    _pad0: u32,

    pub cascade_count: u32,

    _pad1: [u32; 3],
}

impl SceneGPU {
    pub fn create(
        main_camera: MainCameraGPU,
        light_direction: [f32; 3],
        cascade_count: u32,
    ) -> Self {
        Self {
            main_camera,

            light_direction,
            _pad0: 0,

            cascade_count,

            _pad1: [0; 3],
        }
    }
}
