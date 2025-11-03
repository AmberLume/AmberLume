use std::path::Path;

pub trait IOProvider: Send + Sync {
    fn read(&self, path: &Path) -> Option<Vec<u8>>;
}
