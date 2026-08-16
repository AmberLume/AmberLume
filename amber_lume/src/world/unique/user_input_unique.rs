use glam::Vec2;
use input::InputHandler;
use shipyard::Unique;

#[derive(Unique)]
pub struct UserInputUnique {
    pub input: Option<InputHandler>,
    pub pointer_position: Option<Vec2>,
}

impl UserInputUnique {
    pub fn new() -> Self {
        Self {
            input: None,
            pointer_position: None,
        }
    }
}
