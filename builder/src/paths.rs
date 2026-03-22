use std::path::{Path, PathBuf};

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct AlpacaPaths {
    pub name: String,
    pub extension: String,
    
    pub root: PathBuf,
    pub relative: PathBuf,

    pub source: PathBuf,
    pub target: PathBuf,
    pub shared: PathBuf,
}

impl AlpacaPaths {
    pub fn create(
        root: &Path,
        relative: &Path,
        source: &Path,
        target: &Path,
        shared: &Path,
    ) -> Self {
        let name = relative.file_stem().unwrap().to_str().unwrap().to_owned();
        let extension = relative.extension().unwrap().to_str().unwrap().to_owned();
        
        Self {
            name,
            extension,
            
            root: root.to_path_buf(), 
            relative: relative.to_path_buf(),
            
            source: source.to_path_buf(),
            target: target.to_path_buf(),
            shared: shared.to_path_buf(),
        }
    }
    
    pub fn source_file(&self) -> PathBuf {
        self.source.join(&self.relative)
    }
}
