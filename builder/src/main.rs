mod data;
mod pack;
mod utils;

use crate::pack::pack_all;
use alpaca::assembler::pipeline::Pipeline;
use anyhow::Result;
use data::paths::Paths;

fn main() -> Result<()> {
    let paths = Paths::new()?;

    let mut pipeline = Pipeline::new()?;

    pipeline.assemble(&paths.shaders.source, &paths.shaders.target)?;
    pipeline.assemble(&paths.models.source, &paths.models.target)?;

    pack_all(
        "shaders",
        "shaders",
        64,
        &paths.shaders.target,
        &paths.distribution_assets,
    )?;

    pack_all(
        "models",
        "models",
        64,
        &paths.models.target,
        &paths.distribution_assets,
    )?;

    Ok(())
}
