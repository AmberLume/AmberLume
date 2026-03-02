use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use anyhow::{bail, Result};
use log::info;
use crate::build_task::{ConvertKTX2Task, TextureType};
use crate::dispatcher::Dispatcher;
use crate::processors::processor::Processor;

pub struct ConvertKTX2Processor;

impl ConvertKTX2Processor {
    pub fn create() -> Self {
        Self {
            
        }
    }

    fn call_toktx_for(&self, texture_type: &TextureType, input: &Path, output: &Path) -> Result<()> {
        let mut params = vec![
            "--t2",
            "--encode", "uastc",
            "--uastc_quality", "2",
            "--zcmp", "15",
            "--genmipmap",
        ];

        let mut type_params = match texture_type {
            TextureType::Color => {
                vec![
                    "--assign_oetf", "srgb",
                ]
            }
            TextureType::Normal => {
                vec![
                    "--assign_oetf", "linear",
                ]
            }
        };

        params.append(&mut type_params);

        params.push(output.to_str().unwrap());
        params.push(input.to_str().unwrap());

        let status = Command::new("toktx")
            .args(&params)
            .status()?;

        if !status.success() {
            bail!("toktx failed with exit status {}", status);
        }

        Ok(())
    }
}

impl Processor<ConvertKTX2Task> for ConvertKTX2Processor {
    fn process(&self, _dispatcher: Arc<Dispatcher>, task: &ConvertKTX2Task) -> Result<()> {
        let target_path = task.target_path
            .join("textures")
            .join(&task.name)
            .with_extension("ktx2");

        self.call_toktx_for(&task.texture_type, &task.source_path, &target_path)?;

        info!("Converted to KTX2: {}", target_path.display());

        Ok(())
    }
}
