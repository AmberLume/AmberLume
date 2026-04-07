use shipyard::Component;

#[derive(Component)]
pub struct AnimationRenderComponent {
    pub animation_id: u32,
    pub time: f32,

    pub previous_animation_id: u32,
    pub previous_time: f32,
    pub blend_factor: f32,
}
