use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq, serde::Deserialize)]
pub enum ComponentData {
    Character {
        offset: f32,
        auto_step_height: f32,
        max_slope_angle: f32,
        speed: f32,
        push_force: f32,
        jump_velocity: f32,
    },
    Camera {
        fov: f32,
        near: f32,
        far: f32,
    },
}
