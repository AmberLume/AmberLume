mod compile;
mod data;
mod pack;
mod utils;

use crate::compile::compile_shaders;
use crate::pack::pack_all;
use anyhow::Result;
use data::paths::Paths;
use data::variables::Variables;

fn main() -> Result<()> {
    let variables = Variables::new()?;
    let paths = Paths::new(&variables)?;

    compile_shaders(&paths.shaders)?;

    pack_all(
        "shaders",
        "shaders",
        64,
        &paths.shaders.target,
        &paths.distribution_assets,
    )?;

    Ok(())
}
