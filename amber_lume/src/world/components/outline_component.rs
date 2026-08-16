use shipyard::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct OutlineComponent {
    pub color: [f32; 4],
}
