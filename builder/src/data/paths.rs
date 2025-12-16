use crate::data::variables::Variables;
use anyhow::Result;
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
    pub fn new(variables: &Variables) -> Result<Self> {
        let target_module = PathBuf::from(&variables.target_module);

        let distribution = PathBuf::from(&variables.distribution);
        let distribution_assets = PathBuf::from(&variables.distribution_assets);

        let assets = target_module.join("assets");
        let generated_assets = PathBuf::from(&variables.generated_assets);

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
