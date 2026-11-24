use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SceneGPU {
    pub light_direction: [f32; 3],
    pub light_intensity: f32,

    pub light_color: [f32; 3],
    pub ibl_intensity: f32,

    pub cascade_count: u32,
    pub time: f32,

    _pad0: [u32; 2],
}

impl SceneGPU {
    pub fn create(
        light_direction: [f32; 3],
        light_color: [f32; 3],
        light_intensity: f32,
        ibl_intensity: f32,
        cascade_count: u32,
        time: f32,
    ) -> Self {
        Self {
            light_direction,
            light_intensity,

            light_color,
            ibl_intensity,

            cascade_count,
            time,

            _pad0: [0; 2],
        }
    }
}
