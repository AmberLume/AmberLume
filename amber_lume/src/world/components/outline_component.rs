use math::Vec4;
use shipyard::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct OutlineComponent {
    pub color: Vec4,
}
