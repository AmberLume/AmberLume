use shipyard::{EntityId, Unique};

#[derive(Unique, Debug)]
pub struct PlayerControlUnique {
    pub controlled: Option<EntityId>,
}

impl PlayerControlUnique {
    pub fn new() -> Self {
        Self {
            controlled: None,
        }
    }
}
