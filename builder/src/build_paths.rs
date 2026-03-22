use anyhow::Result;
use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug)]
pub struct BuildPaths {
    pub resources: PathBuf,
    pub generated: PathBuf,
    pub prebuild: PathBuf,
    pub alpaca: PathBuf,
    pub shared: PathBuf,
    pub distribution: PathBuf,
}

impl BuildPaths {
    pub fn new() -> Result<Self> {
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
        let project_root_dir = manifest_dir.parent().unwrap();

        let target_module = project_root_dir.join("lume");
        let resources = target_module.join("resources");

        let target = project_root_dir.join("target");
        let generated = target.join("generated");
        let prebuild = generated.join("prebuild");
        let alpaca = generated.join("alpaca");
        let shared = alpaca.join("shared");
        let distribution = target.join("distribution");
    
        create_dir_all(&generated)?;
        create_dir_all(&prebuild)?;
        create_dir_all(&alpaca)?;
        create_dir_all(&shared)?;
        create_dir_all(&distribution)?;
        
        let paths = Self {
            resources,
            generated,
            prebuild,
            alpaca,
            shared,
            distribution,
        };

        Ok(paths)
    }
}
