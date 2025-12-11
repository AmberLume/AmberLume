use anyhow::Result;
use std::env::var;

#[derive(Debug)]
pub struct Variables {
    pub target_module: String,

    pub distribution: String,

    pub generated_assets: String,
    pub distribution_assets: String,
}

impl Variables {
    pub fn new() -> Result<Self> {
        let target_module = var("TARGET_MODULE")?;
        let distribution = var("DISTRIBUTION")?;

        let generated_assets = var("GENERATED_ASSETS")?;
        let distribution_assets = var("DISTRIBUTION_ASSETS")?;

        let variables = Self {
            target_module,

            distribution,

            generated_assets,
            distribution_assets,
        };

        println!("Variables: {:#?}", variables);

        Ok(variables)
    }
}
