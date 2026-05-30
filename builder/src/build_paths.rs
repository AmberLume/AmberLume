use anyhow::Result;
use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug)]
pub struct BuildPaths {
    pub resources: PathBuf,
    pub prebuild: PathBuf,
    pub alpaca: PathBuf,
    pub cache: PathBuf,

    pub distribution: PathBuf,
}

impl BuildPaths {
    pub fn new(module_name: &str) -> Result<Self> {
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
        let project_root_dir = manifest_dir.parent().unwrap();

        let module = project_root_dir.join(module_name);
        
        let target = project_root_dir.join("target");
        let generated = target.join("generated");

        let resources = module.join("resources");
        let prebuild = generated.join("prebuild");
        let alpaca = generated.join("alpaca");
        let cache = generated.join(".builder_cache.bin");

        let distribution = target.join("distribution");
    
        create_dir_all(&generated)?;
        create_dir_all(&alpaca)?;
        create_dir_all(&distribution)?;
        
        let paths = Self {
            resources,
            prebuild,
            alpaca,
            cache,

            distribution,
        };

        Ok(paths)
    }
}
