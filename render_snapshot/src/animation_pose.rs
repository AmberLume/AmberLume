pub struct AnimationPose {
    pub animation_id: u32,
    pub time: f32,

    pub blend_from_animation_id: u32,
    pub blend_from_time: f32,
    pub blend_factor: f32,
}
