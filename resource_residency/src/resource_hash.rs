use ahash::AHasher;
use std::hash::{Hash, Hasher};

#[derive(Hash, Eq, PartialEq)]
pub struct ResourceHash {
    value: u64,
}

impl ResourceHash {
    pub fn of<C: Hash>(config: &C) -> Self {
        let mut hasher = AHasher::default();

        config.hash(&mut hasher);

        Self { value: hasher.finish() }
    }
}
