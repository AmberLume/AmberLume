extern crate core;

mod data;
mod tracing;
mod processors;
mod dispatcher;
mod build_task;
mod build_paths;
mod build_target;

use std::fs::{create_dir_all, read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ::tracing::info;
use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;
use alpaca::packer::alpaca_writer::AlpacaWriter;
use build_paths::BuildPaths;
use crate::build_target::targets_from;
use crate::build_task::{BuildTask, RouteTarget};
use crate::dispatcher::Dispatcher;
use crate::tracing::Tracing;

fn main() -> Result<()> {
    Tracing::initialize();

    let paths = BuildPaths::new("lume")?;

    let dispatcher = Arc::new(Dispatcher::create());

    let mut build_targets = Vec::new();

    build_targets.extend(targets_from(&paths.resources, &["shaders"], &paths.alpaca, &["vert", "frag", "comp"]));
    build_targets.extend(targets_from(&paths.prebuild, &["assets"], &paths.alpaca, &["gltf"]));

    build_targets.into_par_iter().for_each(|build_target| {
        info!("Working on: {:?}...", build_target.relative_full());

        dispatcher.clone().dispatch(BuildTask::RouteTarget(RouteTarget { build_target }));
    });

    dispatcher.wait_all();

    pack_all(&paths)?;

    Ok(())
}

fn pack_all(paths: &BuildPaths) -> Result<()> {
    let mut scenes = Vec::new();
    let mut meshes = Vec::new();
    let mut skeletons = Vec::new();
    let mut animations = Vec::new();
    let mut physical_bodies = Vec::new();
    let mut materials = Vec::new();
    let mut shaders = Vec::new();
    let mut textures = Vec::new();

    let source_path = &paths.alpaca;

    WalkDir::new(&source_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .for_each(|entry| {
            let entry_path = entry.path().to_path_buf();

            let relative_path = entry_path.strip_prefix(&source_path).unwrap().to_path_buf();

            let extension = relative_path.extension().unwrap().to_str().unwrap();

            match extension {
                "SCENE" => scenes.push(relative_path),
                "MESH" => meshes.push(relative_path),
                "SKELETON" => skeletons.push(relative_path),
                "ANIMATION" => animations.push(relative_path),
                "PHYSICAL_BODY" => physical_bodies.push(relative_path),
                "MATERIAL" => materials.push(relative_path),
                "spv" => shaders.push(relative_path),
                "ktx2" => textures.push(relative_path),
                _ => { }
            }
        });

    let target_path = paths.distribution.join("assets");

    create_dir_all(&target_path)?;

    let mut scenes_alpaca = AlpacaWriter::create("scenes", &target_path, 32)?;
    pack_files(&mut scenes_alpaca, &source_path, &scenes)?;

    let mut meshes_alpaca = AlpacaWriter::create("meshes", &target_path, 64)?;
    pack_files(&mut meshes_alpaca, &source_path, &meshes)?;

    let mut skeletons_alpaca = AlpacaWriter::create("skeletons", &target_path, 64)?;
    pack_files(&mut skeletons_alpaca, &source_path, &skeletons)?;

    let mut animations_alpaca = AlpacaWriter::create("animations", &target_path, 64)?;
    pack_files(&mut animations_alpaca, &source_path, &animations)?;

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
