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
use ::tracing::info;
use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;
use alpaca::packer::alpaca_writer::AlpacaWriter;
use build_paths::BuildPaths;
use crate::build_task::{BuildTask, SeedFileTask};
use crate::dispatcher::Dispatcher;
use crate::paths::AlpacaPaths;
use crate::tracing::Tracing;

pub struct BuildTarget {
    pub extension: String,

    pub source: PathBuf,
    pub relative: PathBuf,

    pub target: PathBuf,
}

fn main() -> Result<()> {
    Tracing::initialize();

    let paths = BuildPaths::new()?;

    let shader_targets = collect_targets_from(&paths.resources.join("shaders"), &paths.alpaca.join("shaders"), &["vert", "frag", "comp"]);
    let assets_targets = collect_targets_from(&paths.prebuild.join("assets"), &paths.alpaca.join("assets"), &["gltf"]);

    let mut targets = Vec::with_capacity(shader_targets.len() + assets_targets.len());
    targets.extend(shader_targets);
    targets.extend(assets_targets);

    let dispatcher = Arc::new(Dispatcher::create());

    targets.into_par_iter().for_each(|target| {
        info!("Working on: {}...", target.relative.display());

        let paths = AlpacaPaths::create(
            &paths.prebuild,
            &target.relative,
            &target.source,
            &target.target,
            &paths.shared,
        );

        dispatcher.clone().dispatch(BuildTask::SeedFile(SeedFileTask { paths }));
    });

    dispatcher.wait_all();

    pack_all(&paths)?;

    Ok(())
}

fn pack_all(paths: &BuildPaths) -> Result<()> {
    let mut scenes = Vec::new();
    let mut meshes = Vec::new();
    let mut physical_bodies = Vec::new();
    let mut materials = Vec::new();
    let mut shaders = Vec::new();
    let mut textures = Vec::new();

    let root = paths.generated.join("alpaca");

    WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .for_each(|entry| {
            let entry_path = entry.path().to_path_buf();

            let relative_path = entry_path.strip_prefix(&root).unwrap().to_path_buf();

            let extension = relative_path.extension().unwrap().to_str().unwrap();

            match extension {
                "SCENE" => scenes.push(relative_path),
                "MESH" => meshes.push(relative_path),
                "PHYSICAL_BODY" => physical_bodies.push(relative_path),
                "MATERIAL" => materials.push(relative_path),
                "spv" => shaders.push(relative_path),
                "ktx2" => textures.push(relative_path),
                _ => { }
            }
        });

    let source_path = paths.generated.join("alpaca");
    let target_path = paths.distribution.join("assets");

    create_dir_all(&target_path)?;

    let mut scenes_alpaca = AlpacaWriter::create("scenes", &target_path, 32)?;
    pack_files(&mut scenes_alpaca, &source_path, &scenes)?;

    let mut meshes_alpaca = AlpacaWriter::create("meshes", &target_path, 64)?;
    pack_files(&mut meshes_alpaca, &source_path, &meshes)?;

    let mut physical_bodies_alpaca = AlpacaWriter::create("physical_bodies", &target_path, 64)?;
    pack_files(&mut physical_bodies_alpaca, &source_path, &physical_bodies)?;

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
    source: &Path,
    target: &Path,
    extensions: &[&str],
) -> Vec<BuildTarget> {
    WalkDir::new(&source)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry.path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| extensions.contains(&e))
                .unwrap_or(false)
        })
        .map(|entry| {
            let source = source.to_path_buf();
            let source_path = entry.path().to_path_buf();
            let relative = source_path.strip_prefix(&source).unwrap().to_path_buf();
            let extension = relative.extension().unwrap().to_str().unwrap().to_string();

            let target = target.join(&relative.parent().unwrap());

            BuildTarget {
                extension,

                source,
                relative,

                target,
            }
        })
        .collect()
}
