use amber_lume::platform_providers::io_provider::IOProvider;
use std::env::current_dir;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct DesktopIOProvider {}

impl DesktopIOProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl IOProvider for DesktopIOProvider {
    fn list_files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = Vec::new();

        let current_dir = current_dir().unwrap();
        let assets_dir = current_dir.join("assets");

        for entry in WalkDir::new(&assets_dir).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();

                files.push(path)
            }
        }

        files
    }
}
