use anyhow::Result;
use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug)]
pub struct BuildPaths {
    pub source_assets: PathBuf,
    pub generated: PathBuf,
    pub distribution: PathBuf,
}

impl BuildPaths {
    pub fn new() -> Result<Self> {
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
        let project_root_dir = manifest_dir.parent().unwrap();

        let target_module = project_root_dir.join("../../lume");
        let source_assets = target_module.join("assets");

        let target = project_root_dir.join("../../target");
        let generated = target.join("generated");
        let distribution = target.join("distribution");
    
        create_dir_all(&generated)?;
        create_dir_all(&distribution)?;
        
        let paths = Self {
            source_assets,
            generated,
            distribution,
        };

        println!("Paths: {:#?}", paths);

        Ok(paths)
    }
}
