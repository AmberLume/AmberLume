use glam::Mat4;
use shipyard::Unique;

#[derive(Unique, Debug)]
pub struct WorldCameraUnique {
    pub projection_matrix: Mat4,
}

impl WorldCameraUnique {
    pub fn new() -> Self {
        Self {
            projection_matrix: Mat4::IDENTITY,
        }
    }
}
