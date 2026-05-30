use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use anyhow::{bail, Result};
use tracing::info;
use crate::build_task::{ConvertKTX2Task, TextureType};
use crate::cache::Cache;
use crate::dispatcher::Dispatcher;
use crate::processors::processor::Processor;

pub struct ConvertKTX2Processor {
    cache: Arc<Cache>,
}

impl ConvertKTX2Processor {
    pub fn create(
        cache: Arc<Cache>,
    ) -> Self {
        Self {
            cache,
        }
    }

    fn call_toktx_for(&self, texture_type: &TextureType, input: &Path, output: &Path) -> Result<()> {
        let mut params = vec![
            "--t2",
            "--encode", "uastc",
            "--zcmp", "15",
            "--genmipmap",
        ];

        let mut type_params = match texture_type {
            TextureType::Color => {
                vec![
                    "--assign_oetf", "srgb",
                    "--uastc_quality", "2",
                ]
            }
            TextureType::Normal => {
                vec![
                    "--assign_oetf", "linear",
                    "--uastc_quality", "2",
                ]
            }
            TextureType::OcclusionRoughnessMetallic => {
                vec![
                    "--assign_oetf", "linear",
                    "--uastc_quality", "2",
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
        let target_path = task.build_target.destination.join(&task.resource_key);
        let origin_key = task.build_target.entry.to_string_lossy().into_owned();

        if !target_path.exists() {
            info!("Converting to KTX2: {:?}", target_path);

            self.call_toktx_for(&task.texture_type, &task.source, &target_path)?;
        } else {
            info!("Cached KTX2 {:?}", target_path);
        }

        self.cache.record_output(&origin_key, target_path.to_string_lossy().into_owned());

        Ok(())
    }
}
