mod data;
mod pack;
mod utils;

use crate::pack::pack_all;
use alpaca::assembler::pipeline::Pipeline;
use anyhow::Result;
use data::paths::Paths;

fn main() -> Result<()> {
    let paths = Paths::new()?;

    let mut pipeline = Pipeline::new(&paths.source_assets)?;

    pipeline.assemble(&paths.source_assets, &paths.generated)?;

    pack_all(
        "assets",
        64,
        &paths.generated,
        &paths.distribution.join("assets"),
    )?;

    Ok(())
}
