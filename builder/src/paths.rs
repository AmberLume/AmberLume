use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct Paths {
    pub name: String,
    pub extension: String,
    
    pub relative: PathBuf,

    pub source: PathBuf,
    pub target: PathBuf,
}

impl Paths {
    pub fn create(
        relative: &Path,
        source: &Path,
        target: &Path,
    ) -> Self {
        let name = relative.file_stem().unwrap().to_str().unwrap().to_owned();
        let extension = relative.extension().unwrap().to_str().unwrap().to_owned();
        
        Self {
            name,
            extension,
            
            relative: relative.to_path_buf(),
            
            source: source.to_path_buf(),
            target: target.to_path_buf(),
        }
    }
    
    pub fn source_file(&self) -> PathBuf {
        self.source.join(&self.relative)
    }
}
