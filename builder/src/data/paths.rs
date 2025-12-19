use anyhow::Result;
use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Paths {
    pub distribution_assets: PathBuf,

    pub shaders: ResourcePaths,
    pub models: ResourcePaths,
}

#[derive(Debug)]
pub struct ResourcePaths {
    pub source: PathBuf,
    pub target: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
        let project_root_dir = manifest_dir.parent().unwrap();

        let target_module = project_root_dir.join("lume");
        let assets = target_module.join("assets");

        let target = project_root_dir.join("target");
        let distribution = target.join("distribution");
        let distribution_assets = distribution.join("assets");

        let generated = target.join("generated");
        let generated_assets = generated.join("assets");

        let shaders_resource_paths =
            Self::create_resource_paths("shaders", &assets, &generated_assets)?;
        let models_resource_paths =
            Self::create_resource_paths("models", &assets, &generated_assets)?;

        create_dir_all(&distribution)?;
        create_dir_all(&distribution_assets)?;

        let paths = Self {
            distribution_assets,

            shaders: shaders_resource_paths,
            models: models_resource_paths,
        };

        println!("Paths: {:#?}", paths);

        Ok(paths)
    }

    fn create_resource_paths(
        name: &str,
        source: &PathBuf,
        target: &PathBuf,
    ) -> Result<ResourcePaths> {
        let source = source.join(name);
        let target = target.join(name);

        create_dir_all(&target)?;

        Ok(ResourcePaths { source, target })
    }
}
