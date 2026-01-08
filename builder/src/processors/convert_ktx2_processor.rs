use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use anyhow::{bail, Result};
use log::info;
use crate::build_task::ConvertKTX2Task;
use crate::dispatcher::Dispatcher;
use crate::processors::processor::Processor;

pub struct ConvertKTX2Processor;

impl ConvertKTX2Processor {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn call_toktx_for(&self, is_srgb: bool, input: &Path, output: &Path) -> Result<()> {
        let oetf = if is_srgb { "srgb" } else { "linear" };

        let status = Command::new("toktx")
            .args(&[
                "--t2",
                "--encode", "uastc",
                "--uastc_quality", "2",
                "--zcmp", "15",
                "--assign_oetf", oetf,
                "--genmipmap",
                // "--lower_left_maps_to_s0t0",
                output.to_str().unwrap(),
                input.to_str().unwrap(),
            ])
            .status()?;

        if !status.success() {
            bail!("toktx failed with exit status {}", status);
        }

        Ok(())
    }
}

impl Processor<ConvertKTX2Task> for ConvertKTX2Processor {
    fn process(&self, _dispatcher: Arc<Dispatcher>, task: &ConvertKTX2Task) -> Result<()> {
        let target_path = task.target_path.join("textures").join(&task.name).with_extension("ktx2");

        self.call_toktx_for(true, &task.source_path, &target_path)?;

        info!("Converted to KTX2: {}", target_path.display());

        Ok(())
    }
}
