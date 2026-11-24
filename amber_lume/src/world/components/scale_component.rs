use math::Vec3;
use shipyard::Component;

#[derive(Component, Debug)]
pub struct ScaleComponent {
    pub scale: Vec3,
}
