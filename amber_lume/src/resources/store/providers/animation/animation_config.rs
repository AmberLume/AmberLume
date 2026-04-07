use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum AnimationConfig {
    Alpaca {
        resource_key: String,
    },
}

impl Hash for AnimationConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Alpaca { resource_key } => {
                0.hash(state);

                resource_key.hash(state);
            }
        }
    }
}
