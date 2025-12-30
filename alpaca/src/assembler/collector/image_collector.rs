use crate::assembler::collector::collector::ResourceCollector;
use crate::assembler::key_generator::ResourceKeyGenerator;
use crate::data::common::image_data::ImageData;
use anyhow::{Context, Result, bail};
use gltf::Image;
use gltf::buffer::Data;
use gltf::image::Source;
use image::{RgbaImage, load_from_memory};
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tempdir::TempDir;

pub struct ImageCollector {
    key_generator: Arc<ResourceKeyGenerator>,

    resources: HashMap<String, ImageData>,
}

impl ImageCollector {
    pub fn create(key_generator: Arc<ResourceKeyGenerator>) -> Self {
        Self {
            key_generator,

            resources: HashMap::new(),
        }
    }

    fn compress_to_bc7(is_srgb: bool, image: &RgbaImage) -> Result<Vec<u8>> {
        let temp_dir = TempDir::new("alpaca.ktx2")?;

        let input_file = temp_dir.path().join("input.png");
        let output_file = temp_dir.path().join("output.ktx2");

        image.save(&input_file)?;
        Self::call_toktx_for(is_srgb, &input_file, &output_file)?;

        let ktx2_data = read(&output_file)?;

        Ok(ktx2_data)
    }

    fn call_toktx_for(is_srgb: bool, input: &Path, output: &Path) -> Result<()> {
        let oetf = if is_srgb { "srgb" } else { "linear" };

        let status = Command::new("toktx")
            .args(&[
                "--t2",
                "--encode", "uastc",
                "--uastc_quality", "2",
                "--zcmp", "15",
                "--assign_oetf", oetf,
                "--genmipmap",
                "--lower_left_maps_to_s0t0",
                output.to_str().unwrap(),
                input.to_str().unwrap(),
            ])
            .status()
            .context("failed to execute toktx")?;

        if !status.success() {
            bail!("toktx failed with exit status {}", status);
        }

        Ok(())
    }
}

pub struct ImageResource<'a> {
    pub image: Image<'a>,

    pub local_path: &'a Path,
    
    pub is_srgb: bool,

    pub buffers: &'a [Data],
}

impl ResourceCollector for ImageCollector {
    type Input<'a> = ImageResource<'a>;

    type Output = ImageData;

    fn collect<'a>(&mut self, input: &Self::Input<'a>) -> Result<String> {
        let name = input
            .image
            .name()
            .expect("Image names are required")
            .to_owned();
        let key = input.local_path.join(self.key_generator.get_next_key()).with_extension("ktx2").to_str().unwrap().to_string();
        println!("Collecting image '{}' as '{}'...", name, key);

        let raw_data = match input.image.source() {
            Source::View { view, .. } => {
                let buffer_data = &input.buffers[view.buffer().index()];
                let start = view.offset();
                let end = start + view.length();

                buffer_data[start..end].to_vec()
            }
            Source::Uri { uri, .. } => read(uri)?,
        };

        let decoded = load_from_memory(&raw_data).context("Failed to decode image")?;

        let rgba = decoded.to_rgba8();
        let width = rgba.width();
        let height = rgba.height();

        let bc7_data = Self::compress_to_bc7(input.is_srgb, &rgba)?;

        self.resources.insert(
            key.clone(),
            ImageData {
                width,
                height,
                data: bc7_data,
            },
        );

        Ok(key)
    }

    fn get_resources(&self) -> Vec<(String, &Self::Output)> {
        self.resources
            .iter()
            .map(|(key, value)| (key.clone(), value))
            .collect()
    }
    
    fn reset(&mut self) {
        self.resources.clear();
    }
}
