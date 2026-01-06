use shipyard::Unique;
use crate::input_handler::input_event::KeyEvent;
use crate::input_handler::input_state::InputState;

#[derive(Unique, Debug)]
pub struct UserInputUnique {
    pub events: Vec<KeyEvent>,
    
    pub state: InputState,
}

impl UserInputUnique {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            
            state: InputState::create(),
        }
    }
}
