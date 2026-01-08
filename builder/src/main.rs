extern crate core;

mod data;
mod tracing;
mod processors;
mod dispatcher;
mod build_task;
mod paths;
mod gltf_file;
mod aabb_utils;
mod build_paths;

use std::fs::{create_dir_all, read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use log::info;
use rayon::prelude::*;
use walkdir::WalkDir;
use alpaca::packer::alpaca_writer::AlpacaWriter;
use build_paths::BuildPaths;
use crate::build_task::{BuildTask, SeedFileTask};
use crate::dispatcher::Dispatcher;
use crate::paths::Paths;
use crate::tracing::Tracing;

pub struct BuildTarget {
    pub extension: String,

    pub source_root: PathBuf,
    pub relative_path: PathBuf,

    pub generated_path: PathBuf,
}

fn main() -> Result<()> {
    Tracing::initialize();

    let paths = BuildPaths::new()?;

    let shader_targets = collect_targets_from("shaders", &paths.source_assets, &paths.generated);
    let scene_targets = collect_targets_from("scenes", &paths.source_assets, &paths.generated);

    let mut targets = Vec::with_capacity(shader_targets.len() + scene_targets.len());
    targets.extend(shader_targets);
    targets.extend(scene_targets);

    let dispatcher = Arc::new(Dispatcher::create());

    targets.into_par_iter().for_each(|target| {
        info!("Working on: {}...", target.relative_path.display());

        let paths = Paths::create(
            &target.relative_path,
            &target.source_root,
            &target.generated_path
        );

        dispatcher.clone().dispatch(BuildTask::SeedFile(SeedFileTask { paths }));
    });

    dispatcher.wait_all();

    pack_all(&paths)?;

    Ok(())
}

fn pack_all(paths: &BuildPaths) -> Result<()> {
    let mut scenes = Vec::new();
    let mut models = Vec::new();
    let mut materials = Vec::new();
    let mut shaders = Vec::new();
    let mut textures = Vec::new();

    WalkDir::new(&paths.generated)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .for_each(|entry| {
            let entry_path = entry.path().to_path_buf();

            let relative_path = entry_path.strip_prefix(&paths.generated).unwrap().to_path_buf();

            let extension = relative_path.extension().unwrap().to_str().unwrap();

            match extension {
                "scene" => scenes.push(relative_path),
                "model" => models.push(relative_path),
                "material" => materials.push(relative_path),
                "spv" => shaders.push(relative_path),
                "ktx2" => textures.push(relative_path),
                _ => { }
            }
        });

    let source_path = paths.generated.clone();
    let target_path = paths.distribution.join("assets");

    create_dir_all(&target_path)?;

    let mut scenes_alpaca = AlpacaWriter::create("scenes", &target_path, 32)?;
    pack_files(&mut scenes_alpaca, &source_path, &scenes)?;

    let mut models_alpaca = AlpacaWriter::create("models", &target_path, 64)?;
    pack_files(&mut models_alpaca, &source_path, &models)?;

    let mut materials_alpaca = AlpacaWriter::create("materials", &target_path, 64)?;
    pack_files(&mut materials_alpaca, &source_path, &materials)?;

    let mut shaders_alpaca = AlpacaWriter::create("shaders", &target_path, 64)?;
    pack_files(&mut shaders_alpaca, &source_path, &shaders)?;

    let mut textures_alpaca = AlpacaWriter::create("textures", &target_path, 64)?;
    pack_files(&mut textures_alpaca, &source_path, &textures)?;

    Ok(())
}

fn pack_files(alpaca: &mut AlpacaWriter, source_path: &Path, files: &Vec<PathBuf>) -> Result<()> {
    files.iter().for_each(|path| {
        let key = path.to_str().unwrap();
        let path = source_path.join(key);
        let data = read(path).unwrap();

        alpaca.push(&key, &data).unwrap();
    });

    alpaca.pack()?;

    Ok(())
}

fn collect_targets_from(
    directory: &str,
    source_root: &Path,
    generated_root: &Path,
) -> Vec<BuildTarget> {
    let target = source_root.join(directory);

    WalkDir::new(&target)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            let source_path = entry.path().to_path_buf();

            let relative_path = source_path.strip_prefix(&source_root).unwrap().to_path_buf();

            let extension = relative_path.extension().unwrap().to_str().unwrap().to_string();

            let generated_path = generated_root.join(&relative_path.parent().unwrap());

            BuildTarget {
                extension,

                source_root: source_root.to_path_buf(),
                relative_path,

                generated_path,
            }
        })
        .collect()
}
