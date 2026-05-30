extern crate core;

mod tracing;
mod processors;
mod dispatcher;
mod build_task;
mod build_paths;
mod build_target;
mod manifest;
mod cache;

use std::collections::HashMap;
use std::fs::{create_dir_all, read, remove_file};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ::tracing::{info, warn};
use anyhow::Result;
use rayon::prelude::*;
use walkdir::WalkDir;
use alpaca::packer::alpaca_writer::AlpacaWriter;
use build_paths::BuildPaths;
use crate::build_target::targets_from;
use crate::build_task::{BuildTask, RouteTarget};
use crate::cache::Cache;
use crate::dispatcher::Dispatcher;
use crate::manifest::{generate_manifest, Category};
use crate::tracing::Tracing;

fn main() -> Result<()> {
    Tracing::initialize();

    let paths = BuildPaths::new("lume/core")?;

    let cache = Arc::new(Cache::load(&paths.cache).unwrap_or_else(|_| Cache::new()));
    let dispatcher = Arc::new(Dispatcher::create(cache.clone()));

    let mut build_targets = Vec::new();

    build_targets.extend(targets_from(&paths.resources, &["shaders"], &paths.alpaca, &["vert", "frag", "comp"]));
    build_targets.extend(targets_from(&paths.prebuild, &["assets"], &paths.alpaca, &["gltf"]));

    build_targets.into_par_iter().for_each(|build_target| {
        info!("Working on: {:?}...", build_target.relative_full());

        dispatcher.clone().dispatch(BuildTask::RouteTarget(RouteTarget { build_target }));
    });

    dispatcher.wait_all();

    let live_outputs = cache.live_outputs();

    let mut removed_files = 0;
    for entry in WalkDir::new(&paths.alpaca).into_iter().filter_map(|entry| entry.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();

        if live_outputs.contains(path) {
            continue;
        }

        match remove_file(path) {
            Ok(()) => {
                info!("Removed orphan output {}", path.display());
                removed_files += 1;
            }
            Err(err) => warn!("Failed to remove orphan output {}: {}", path.display(), err),
        }
    }

    pack_all(&paths)?;

    let (untouched, touched, new) = cache.summary();
    info!(
        "Cache: untouched={}, touched={}, new={} ({} orphan files removed)",
        untouched.len(), touched.len(), new.len(), removed_files,
    );

    cache.save(&paths.cache)?;

    Ok(())
}

struct CategorySpec {
    module: &'static str,
    handle: &'static str,
    extension: &'static str,
    align: u64,
}

const CATEGORY_SPECS: &[CategorySpec] = &[
    CategorySpec { module: "scenes", handle: "SceneResource", extension: "SCENE", align: 32 },
    CategorySpec { module: "meshes", handle: "MeshResource", extension: "MESH", align: 64 },
    CategorySpec { module: "skeletons", handle: "SkeletonResource", extension: "SKELETON", align: 64 },
    CategorySpec { module: "animations", handle: "AnimationResource", extension: "ANIMATION", align: 64 },
    CategorySpec { module: "physical_bodies", handle: "PhysicalBodyResource", extension: "PHYSICAL_BODY", align: 64 },
    CategorySpec { module: "materials", handle: "MaterialResource", extension: "MATERIAL", align: 64 },
    CategorySpec { module: "shaders", handle: "ShaderResource", extension: "spv", align: 64 },
    CategorySpec { module: "textures", handle: "TextureResource", extension: "ktx2", align: 64 },
];

fn pack_all(paths: &BuildPaths) -> Result<()> {
    let source_path = &paths.alpaca;

    let mut buckets: HashMap<&'static str, Vec<PathBuf>> = HashMap::new();

    WalkDir::new(&source_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .for_each(|entry| {
            let entry_path = entry.path().to_path_buf();

            let relative_path = entry_path.strip_prefix(&source_path).unwrap().to_path_buf();

            let extension = relative_path.extension().unwrap().to_str().unwrap();

            if let Some(spec) = CATEGORY_SPECS.iter().find(|spec| spec.extension == extension) {
                buckets.entry(spec.module).or_default().push(relative_path);
            }
        });

    let target_path = paths.distribution.join("assets");

    create_dir_all(&target_path)?;

    let mut categories = Vec::new();

    for spec in CATEGORY_SPECS {
        let mut files = buckets.remove(spec.module).unwrap_or_default();
        files.sort();

        let mut alpaca = AlpacaWriter::create(spec.module, &target_path, spec.align)?;
        pack_files(&mut alpaca, &source_path, &files)?;

        categories.push(Category {
            module: spec.module,
            handle: spec.handle,
            files,
        });
    }

    let generated_dir = paths.alpaca.parent().unwrap();
    generate_manifest(generated_dir, &categories)?;

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
