use std::sync::atomic::{AtomicU64, Ordering};

pub struct ResourceKeyGenerator {
    next_key: AtomicU64,
}

impl ResourceKeyGenerator {
    pub fn create() -> Self {
        Self {
            next_key: AtomicU64::new(0),
        }
    }

    pub fn get_next_key(&self) -> String {
        let next_key = self.next_key.fetch_add(1, Ordering::Relaxed);

        next_key.to_string()
    }
}
