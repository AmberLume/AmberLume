use std::path::PathBuf;
use crate::build_target::BuildTarget;
use crate::data::resource_key::ResourceKey;
use crate::processors::assets::link_utils::LinkExtras;

pub fn resource_key(build_target: &BuildTarget, name: &str, extension: &str) -> ResourceKey {
    let key = build_target.relative.join(name).with_added_extension(extension);

    ResourceKey {
        value: key.to_str().unwrap().to_string(),
    }
}

pub fn resource_key_from_build_target(build_target: &BuildTarget, extension: &str) -> ResourceKey {
    let key = build_target.relative
        .join(&build_target.name)
        .with_added_extension(&build_target.extension)
        .with_added_extension(extension);

    ResourceKey {
        value: key.to_str().unwrap().to_string(),
    }
}

pub fn resource_key_from_extras(build_target: &BuildTarget, link_extras: &LinkExtras, extension: &str) -> ResourceKey {
    let relative = PathBuf::from(&link_extras.source_gltf);
    let build_target = build_target.to_relative(&relative).unwrap();

    let name_path = PathBuf::from(&link_extras.source_collection);
    let name = name_path.file_stem().unwrap().to_str().unwrap();

    resource_key(&build_target, &name, &extension)
}
