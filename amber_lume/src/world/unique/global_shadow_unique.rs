use glam::Vec3;
use shipyard::Unique;

#[derive(Unique, Debug)]
pub struct GlobalShadowUnique {
    pub direction: Vec3,
}

impl GlobalShadowUnique {
    pub fn new() -> Self {
        Self {
            direction: Vec3::new(0.0, -1.0, 0.0),
        }
    }
}
