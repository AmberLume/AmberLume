use crate::compiler::pipeline::CompilationResult;
use crate::compiler::resource_compiler::ResourceCompiler;
use anyhow::{Result, bail};
use gltf::Gltf;
use gltf::image::Source;
use std::path::Path;
use std::process::Command;

pub struct ModelCompiler {}

impl ModelCompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl ResourceCompiler for ModelCompiler {
    fn extensions(&self) -> &[&str] {
        &["gltf"]
    }

    fn compile(
        &self,
        _name: &String,
        src: &Path,
        dst_dir: &Path,
    ) -> Result<Vec<CompilationResult>> {
        let gltf = Gltf::open(src)?;

        let mut results = Vec::new();

        for image in gltf.images() {
            let name = match image.source() {
                Source::View { .. } => {
                    bail!("View images are not supported. Use Uri images instead.");
                }
                Source::Uri { uri, .. } => {
                    let image_path = src.parent().unwrap().join(uri);
                    let result_image_path = dst_dir.join(uri);

                    convert_to_ktx2(&image_path, &result_image_path)?;

                    uri
                }
            };

            results.push(CompilationResult {
                name: name.to_string(),
            });
        }

        Ok(results)
    }
}

pub fn convert_to_ktx2(input: &Path, output: &Path) -> Result<()> {
    let status = Command::new("toktx")
        .args([
            "--t2",
            "--uastc",
            "4",
            "--genmipmap",
            &format!("{}.ktx2", output.to_str().unwrap()),
            input.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        eprintln!("toktx failed with code {:?}", status.code());
    }

    Ok(())
}
