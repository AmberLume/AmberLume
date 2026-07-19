use input::InputHandler;
use shipyard::Unique;

#[derive(Unique)]
pub struct UserInputUnique {
    pub input: Option<InputHandler>,
}

impl UserInputUnique {
    pub fn new() -> Self {
        Self {
            input: None,
        }
    }
}
