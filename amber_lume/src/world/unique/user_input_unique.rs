use shipyard::Unique;
use crate::input_handler::input_frame::InputFrame;

#[derive(Unique, Debug)]
pub struct UserInputUnique {
    pub input_frame: InputFrame,
}

impl UserInputUnique {
    pub fn new() -> Self {
        Self {
            input_frame: InputFrame::create(),
        }
    }
}
