use std::path::PathBuf;

pub trait IOProvider: Send + Sync {
    fn list_files(&self) -> Vec<PathBuf>;
}
