use shipyard::Component;

#[derive(Component)]
pub struct AnimationRenderComponent {
    pub animation_id: u32,
    pub time: f32,
}
