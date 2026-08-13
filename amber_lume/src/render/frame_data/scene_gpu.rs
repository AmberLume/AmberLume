use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use gpu::{ViewMatrix, ViewProjectionMatrix};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct MainCameraGPU {
    pub view_projection: [[f32; 4]; 4],
    pub previous_view_projection: [[f32; 4]; 4],
    pub jittered_view_projection: [[f32; 4]; 4],
    pub inverse_view_projection: [[f32; 4]; 4],
    pub inverse_jittered_view_projection: [[f32; 4]; 4],

    pub view: [[f32; 4]; 4],

    pub position: [f32; 3],
    _pad0: u32,

    pub near: f32,
    pub far: f32,
    pub ndc_to_view_mul: [f32; 2],
    pub ndc_to_view_add: [f32; 2],
    pub mip_bias: f32,
    _pad1: u32,
}

impl MainCameraGPU {
    pub fn new(
        view_projection: &ViewProjectionMatrix,
        previous_view_projection: &ViewProjectionMatrix,
        jittered_view_projection: &ViewProjectionMatrix,
        inverse_view_projection: &ViewProjectionMatrix,
        inverse_jittered_view_projection: &ViewProjectionMatrix,
        view: &ViewMatrix,
        position: Vec3,
        near: f32,
        far: f32,
        ndc_to_view_mul: Vec2,
        ndc_to_view_add: Vec2,
        mip_bias: f32,
    ) -> Self {
        Self {
            view_projection: view_projection.value.to_cols_array_2d(),
            previous_view_projection: previous_view_projection.value.to_cols_array_2d(),
            jittered_view_projection: jittered_view_projection.value.to_cols_array_2d(),
            inverse_view_projection: inverse_view_projection.value.to_cols_array_2d(),
            inverse_jittered_view_projection: inverse_jittered_view_projection.value.to_cols_array_2d(),

            view: view.value.to_cols_array_2d(),

            position: position.to_array(),
            _pad0: 0,

            near,
            far,
            ndc_to_view_mul: ndc_to_view_mul.to_array(),
            ndc_to_view_add: ndc_to_view_add.to_array(),
            mip_bias,
            _pad1: 0,
        }
    }
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneGPU {
    pub main_camera: MainCameraGPU,

    pub light_direction: [f32; 3],
    pub light_intensity: f32,

    pub light_color: [f32; 3],
    pub ibl_intensity: f32,

    pub cascade_count: u32,
    pub time: f32,

    _pad1: [u32; 2],
}

impl SceneGPU {
    pub fn create(
        main_camera: MainCameraGPU,
        light_direction: [f32; 3],
        light_color: [f32; 3],
        light_intensity: f32,
        ibl_intensity: f32,
        cascade_count: u32,
        time: f32,
    ) -> Self {
        Self {
            main_camera,

            light_direction,
            light_intensity,

            light_color,
            ibl_intensity,

            cascade_count,
            time,

            _pad1: [0; 2],
        }
    }
}
