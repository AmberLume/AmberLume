use amber_lume::resources::providers::io_provider::IOProvider;
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

        let assets_root = PathBuf::from("assets");

        for entry in WalkDir::new(&assets_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();

                files.push(path)
            }
        }

        files
    }
}
